import serial
import time

def main():
    print("Start 1200-baud touch")
    ser = serial.Serial("COM8", 1200)
    ser.setDTR(False)
    time.sleep(0.2)
    ser.close()
    time.sleep(0.2)
    print("Done")

if __name__ == "__main__":
    main()