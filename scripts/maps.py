import math
import requests

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

    print(query)

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


def save_polylines_txt(filename, polylines):
    with open(filename, "w") as f:
        for line in polylines:
            for x, y in line:
                f.write(f"{x:.6f} {y:.6f}\n")
            f.write("\n")


if __name__ == "__main__":
    lat = 42.39625701047068
    lon = -71.10866957285928
    side_meters = 10000

    # Davis Square (for demo only)
    lat = 42.396521362143275
    lon = -71.12230238395239
    side_meters = 500

    roads = fetch_osm_roads(lat, lon, side_meters)

    save_polylines_txt("roads.txt", roads)
    print(f"Saved {len(roads)} polylines to roads.txt")
