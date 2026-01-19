import math
import requests

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
      way["highway"~"primary|secondary|residential|footway|path"]["service"!="parking_aisle"]({south},{west},{north},{east});
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


def project_and_crop(polylines, lat0, lon0, side_meters):
    """
    Projects lon/lat to local XY (meters), crops to a square of size side_meters,
    and normalizes to [-1,+1] relative to that square.
    Returns List[List[(x, y)]].
    """
    lat0_rad = math.radians(lat0)
    half_side = side_meters / 2

    # Project lat/lon -> local meters
    projected = []
    for line in polylines:
        proj_line = []
        for lon, lat in line:
            x = (lon - lon0) * math.cos(lat0_rad) * 111_320  # meters
            y = (lat - lat0) * 111_320
            proj_line.append((x, y))
        projected.append(proj_line)

    # Define crop boundaries in meters
    min_x, max_x = -half_side, half_side
    min_y, max_y = -half_side, half_side

    # Clip lines to the square
    cropped = []
    for line in projected:
        clipped_line = [
            (x, y) for x, y in line if min_x <= x <= max_x and min_y <= y <= max_y
        ]
        if len(clipped_line) >= 2:
            cropped.append(clipped_line)

    # Normalize directly to [-1, +1] using side_meters
    normalized = []
    for line in cropped:
        norm_line = [((x / half_side), (y / half_side)) for x, y in line]
        normalized.append(norm_line)

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
    side_meters = 500

    roads = fetch_osm_roads(lat, lon, side_meters)
    norm_roads = project_and_crop(roads, lat, lon, side_meters)

    save_polylines_txt("roads.txt", norm_roads)
    print(f"Saved {len(norm_roads)} polylines to roads.txt")
