import math
import requests
from shapely.geometry import LineString, box

OVERPASS_URL = "https://overpass-api.de/api/interpreter"


def meters_to_degrees(lat, meters):
    lat_rad = math.radians(lat)
    meters_per_deg_lat = 111_320
    meters_per_deg_lon = 111_320 * math.cos(lat_rad)
    return meters / meters_per_deg_lat, meters / meters_per_deg_lon


def fetch_osm_roads(lat, lon, side_meters):
    half = side_meters / 2
    dlat, dlon = meters_to_degrees(lat, half)

    south, north = lat - dlat, lat + dlat
    west, east = lon - dlon, lon + dlon

    query = f"""
    [out:json][timeout:25];
    (
      way["highway"~"motorway|trunk|primary|secondary|tertiary|residential"]({south},{west},{north},{east});
    );
    out geom;
    """

    response = requests.post(OVERPASS_URL, data=query)
    response.raise_for_status()
    data = response.json()

    polylines = []
    for element in data["elements"]:
        if element["type"] != "way" or "geometry" not in element:
            continue
        line = [(pt["lon"], pt["lat"]) for pt in element["geometry"]]
        if len(line) >= 2:
            polylines.append(line)
    return polylines


def project_crop(polylines, lat0, lon0, side_meters):
    lat0_rad = math.radians(lat0)
    half_side = side_meters / 2
    crop_rect = box(-half_side, -half_side, half_side, half_side)

    clipped = []

    for line in polylines:
        if len(line) < 2:
            continue
        projected = [
            ((lon - lon0) * math.cos(lat0_rad) * 111_320, (lat - lat0) * 111_320)
            for lon, lat in line
        ]
        shapely_line = LineString(projected)
        intersection = shapely_line.intersection(crop_rect)

        if intersection.is_empty:
            continue
        if intersection.geom_type == "LineString":
            clipped.append(list(intersection.coords))
        elif intersection.geom_type == "MultiLineString":
            for subline in intersection.geoms:
                if len(subline.coords) >= 2:
                    clipped.append(list(subline.coords))
    return clipped


def merge_connected_lines(lines, tol=1e-3):
    """
    Merge lines with matching endpoints within tolerance.
    """
    merged = []

    while lines:
        line = lines.pop(0)
        changed = True
        while changed:
            changed = False
            for i, other in enumerate(lines):
                if math.dist(line[-1], other[0]) < tol:
                    line.extend(other[1:])
                    lines.pop(i)
                    changed = True
                    break
                elif math.dist(line[-1], other[-1]) < tol:
                    line.extend(reversed(other[:-1]))
                    lines.pop(i)
                    changed = True
                    break
                elif math.dist(line[0], other[-1]) < tol:
                    line = other[:-1] + line
                    lines.pop(i)
                    changed = True
                    break
                elif math.dist(line[0], other[0]) < tol:
                    line = list(reversed(other[1:])) + line
                    lines.pop(i)
                    changed = True
                    break
        merged.append(line)
    return merged


def greedy_order_lines(lines):
    """
    Reorder lines to reduce travel distance between disconnected segments.
    """
    if not lines:
        return []

    ordered = [lines.pop(0)]

    while lines:
        last_point = ordered[-1][-1]
        best_idx = None
        best_dist = float("inf")
        reverse = False

        for i, line in enumerate(lines):
            dist_start = math.dist(last_point, line[0])
            dist_end = math.dist(last_point, line[-1])

            if dist_start < best_dist:
                best_dist, best_idx, reverse = dist_start, i, False
            if dist_end < best_dist:
                best_dist, best_idx, reverse = dist_end, i, True

        next_line = lines.pop(best_idx)
        if reverse:
            next_line.reverse()
        ordered.append(next_line)

    return ordered


def normalize_lines(lines, side_meters):
    half_side = side_meters / 2
    normalized = [[(x / half_side, y / half_side) for x, y in line] for line in lines]
    return normalized


def save_polylines_txt(filename, polylines):
    with open(filename, "w") as f:
        for line in polylines:
            for x, y in line:
                f.write(f"{x:.6f} {y:.6f}\n")
            f.write("\n")


if __name__ == "__main__":
    lat = 42.39625701047068
    lon = -71.10866957285928
    side_meters = 1000

    roads = fetch_osm_roads(lat, lon, side_meters)

    save_polylines_txt("roads.txt", roads)
    print(f"Saved {len(roads)} polylines to roads.txt")
