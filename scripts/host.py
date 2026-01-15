import serial
import json
import time

PORT = "/dev/cu.usbmodem1201"


def main():
    ser = serial.Serial(
        port=PORT,
        baudrate=115200,
        timeout=1,
    )

    time.sleep(2)  # give the device time to reset

    payload = {
        "cmd": "SetIndicatorLight",
        "r": 127,
        "g": 127,
        "b": 127,
    }

    # newline-delimited JSON
    msg = json.dumps(payload) + "\n"

    print("→ sending:", msg.strip())
    ser.write(msg.encode("utf-8"))

    # read response (also newline-delimited)
    response = ser.readline()
    if response:
        print("← received:", response.decode("utf-8", errors="replace").strip())
    else:
        print("← no response")

    ser.close()


if __name__ == "__main__":
    main()
