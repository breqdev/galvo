import math
import requests
from shapely.geometry import LineString, box

OVERPASS_URL = "https://overpass-api.de/api/interpreter"


def meters_to_degrees(lat, meters):
    """
    Convert meters to degrees latitude/longitude at a given latitude.
    """
    lat_rad = math.radians(lat)

    meters_per_deg_lat = 111_320
    meters_per_deg_lon = 111_320 * math.cos(lat_rad)

    return (
        meters / meters_per_deg_lat,
        meters / meters_per_deg_lon,
    )


def fetch_osm_roads(lat, lon, side_meters):
    """
    Fetch OSM road geometry in a square area centered on lat/lon.

    Returns:
        List[List[(lon, lat)]]
    """
    half = side_meters / 2
    dlat, dlon = meters_to_degrees(lat, half)

    south = lat - dlat
    north = lat + dlat
    west = lon - dlon
    east = lon + dlon

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
        if element["type"] != "way":
            continue

        if "geometry" not in element:
            continue

        line = [(pt["lon"], pt["lat"]) for pt in element["geometry"]]

        if len(line) >= 2:
            polylines.append(line)

    return polylines


def project_crop_and_normalize(polylines, lat0, lon0, side_meters):
    """
    Project lat/lon to local meters, clip to a square, and normalize to [-1,+1].
    Uses Shapely for robust clipping.
    """
    lat0_rad = math.radians(lat0)
    half_side = side_meters / 2

    # Crop square in local meters
    crop_rect = box(-half_side, -half_side, half_side, half_side)

    normalized = []

    for line in polylines:
        if len(line) < 2:
            continue

        # Project lon/lat -> local meters
        projected = [
            ((lon - lon0) * math.cos(lat0_rad) * 111_320, (lat - lat0) * 111_320)
            for lon, lat in line
        ]

        shapely_line = LineString(projected)
        clipped = shapely_line.intersection(crop_rect)

        # clipped can be LineString or MultiLineString
        if clipped.is_empty:
            continue
        if clipped.geom_type == "LineString":
            coords = list(clipped.coords)
            normalized.append([(x / half_side, y / half_side) for x, y in coords])
        elif clipped.geom_type == "MultiLineString":
            for subline in clipped.geoms:
                coords = list(subline.coords)
                if len(coords) >= 2:
                    normalized.append(
                        [(x / half_side, y / half_side) for x, y in coords]
                    )

    return normalized


def save_polylines_txt(filename, polylines):
    """
    Save polylines to a simple text file.
    Each line: x y
    Empty line between polylines
    """
    with open(filename, "w") as f:
        for line in polylines:
            for x, y in line:
                f.write(f"{x:.6f} {y:.6f}\n")
            f.write("\n")  # blank line = pen-up / new polyline


if __name__ == "__main__":
    lat = 42.39625701047068
    lon = -71.10866957285928
    side_meters = 1000

    roads = fetch_osm_roads(lat, lon, side_meters)
    norm_roads = project_crop_and_normalize(roads, lat, lon, side_meters)

    save_polylines_txt("roads.txt", norm_roads)
    print(f"Saved {len(norm_roads)} polylines to roads.txt")
