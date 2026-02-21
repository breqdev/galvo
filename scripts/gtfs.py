import requests
import polyline

routes = requests.get(
    "https://api-v3.mbta.com/routes?filter[type]=0,1",
    headers={"X-API-KEY": "e9a2c5b0627544ba8b8a509422368fed"},
).json()["data"]

for route in routes:
    route_patterns = requests.get(
        f"https://api-v3.mbta.com/route_patterns?filter[route]={route['id']}",
        headers={"X-API-KEY": "e9a2c5b0627544ba8b8a509422368fed"},
    ).json()["data"]

    typical_routes = [p for p in route_patterns if p["attributes"]["typicality"] == 1]

    for typical_route in typical_routes:
        representative_trip_id = typical_route["relationships"]["representative_trip"][
            "data"
        ]["id"]
        representative_trip = requests.get(
            f"https://api-v3.mbta.com/trips/{representative_trip_id}",
            headers={"X-API-KEY": "e9a2c5b0627544ba8b8a509422368fed"},
        ).json()["data"]
        shape_id = representative_trip["relationships"]["shape"]["data"]["id"]

        shape = requests.get(
            f"https://api-v3.mbta.com/shapes/{shape_id}",
            headers={"X-API-KEY": "e9a2c5b0627544ba8b8a509422368fed"},
        ).json()["data"]

        print("#" + route["attributes"]["color"])

        line_data = polyline.decode(shape["attributes"]["polyline"])

        for line in line_data:
            print(line[1], line[0])

        print()
