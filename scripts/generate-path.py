import math
from HersheyFonts import HersheyFonts
import serial
import json

# ['futural', 'astrology', 'cursive', 'cyrilc_1', 'cyrillic', 'futuram', 'gothgbt', 'gothgrt', 'gothiceng', 'gothicger', 'gothicita', 'gothitt', 'greek', 'greekc', 'greeks', 'japanese', 'markers', 'mathlow', 'mathupp', 'meteorology', 'music', 'rowmand', 'rowmans', 'rowmant', 'scriptc', 'scripts', 'symbolic', 'timesg', 'timesi', 'timesib', 'timesr', 'timesrb']

thefont = HersheyFonts()
# thefont.load_default_font("gothiceng")
thefont.load_default_font()
thefont.normalize_rendering(0.2)


while True:
    message = input("> ")

    # x, y, distance, pen
    points = []

    for (x1, y1), (x2, y2) in thefont.lines_for_text(message):
        # print(f"{(x1, y1)} -> {(x2, y2)}")
        if len(points) == 0:
            points.append((x1, y1, 1, False))
        else:
            last_x, last_y, _, last_pen = points[-1]
            lifted_pen = not (last_x == x1 and last_y == y1)
            distance = math.sqrt((last_x - x1) ** 2 + (last_y - y1) ** 2)
            if lifted_pen:
                points.append((x1, y1, distance, False))

        distance = math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2)

        points.append((x2, y2, distance, True))

    results = []

    for x, y, distance, pen in points:
        results.append(
            {
                "x": 255 - int(x * 256),
                "y": int(y * 256),
                "delay": int(5000 * distance),
                "red": pen,
                "green": False,
                "blue": False,
            }
        )

    ser = serial.Serial(
        port="/dev/cu.usbmodem1201",
        baudrate=115200,
        timeout=2,
    )

    msg = json.dumps({"cmd": "SetWaveform", "points": results}) + "\n"
    print("sending")
    ser.write(msg.encode("utf-8"))
    response = json.loads(ser.readline())
    if response["success"]:
        print("success!")
    else:
        print("error")
