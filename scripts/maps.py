import math
import requests
import matplotlib.pyplot as plt

OVERPASS_URL = "https://overpass-api.de/api/interpreter"

COLOR_MAP = {
    "motorway":    (0.9, 0.1, 0.1),   # red
    "trunk":       (0.9, 0.4, 0.1),   # orange
    "primary":     (0.9, 0.8, 0.1),   # yellow
    "secondary":   (0.2, 0.7, 0.2),   # green
    "tertiary":    (0.2, 0.5, 0.9),   # blue
    "residential": (0.7, 0.7, 0.7),   # gray
}
DEFAULT_COLOR = (0.5, 0.5, 0.5)


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
        highway_type = element.get("tags", {}).get("highway", "unknown")
        line = [(pt["lon"], pt["lat"]) for pt in element["geometry"]]
        if len(line) >= 2:
            polylines.append((line, highway_type))
    return polylines


def save_polylines_txt(filename, polylines):
    with open(filename, "w") as f:
        for line, highway_type in polylines:
            r, g, b = COLOR_MAP.get(highway_type, DEFAULT_COLOR)
            f.write(f"# {highway_type} {r:.3f} {g:.3f} {b:.3f}\n")
            for x, y in line:
                f.write(f"{x:.6f} {y:.6f}\n")
            f.write("\n")


def visualize(polylines):
    fig, ax = plt.subplots(figsize=(10, 10), facecolor="black")
    ax.set_facecolor("black")
    ax.set_aspect("equal")
    ax.axis("off")

    for line, highway_type in polylines:
        color = COLOR_MAP.get(highway_type, DEFAULT_COLOR)
        xs = [pt[0] for pt in line]
        ys = [pt[1] for pt in line]
        linewidth = {"motorway": 3, "trunk": 2.5, "primary": 2}.get(highway_type, 1)
        ax.plot(xs, ys, color=color, linewidth=linewidth)

    # Legend
    for road_type, color in COLOR_MAP.items():
        ax.plot([], [], color=color, label=road_type, linewidth=2)
    ax.legend(loc="lower right", facecolor="#222", labelcolor="white", framealpha=0.8)

    plt.tight_layout()
    plt.savefig("roads.png", dpi=150, bbox_inches="tight", facecolor="black")
    plt.show()
    print("Saved visualization to roads.png")


if __name__ == "__main__":
    lat = 42.396521362143275
    lon = -71.12230238395239
    side_meters = 1000

    roads = fetch_osm_roads(lat, lon, side_meters)
    save_polylines_txt("roads.txt", roads)
    print(f"Saved {len(roads)} polylines to roads.txt")

    visualize(roads)