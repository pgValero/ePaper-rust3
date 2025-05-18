from PIL import Image, ImageOps

WIDTH = 800
HEIGHT = 480

black = Image.open("image.png")
black = ImageOps.pad(image=black, size=(WIDTH, HEIGHT), color="white").convert("1")
red = Image.open("imagen.jpg")
red = ImageOps.pad(image=red, size=(WIDTH, HEIGHT), color="white").convert("1")

black_bytes = bytearray(black.tobytes('raw'))
red_bytes = bytearray(red.tobytes('raw'))

for i in range(len(red_bytes)):
    red_bytes[i] ^= 0xFF
    # black_bytes[i] = 0xFF


buf = black_bytes + red_bytes

print(len(buf))

from requests import request

URL = "http://192.168.1.139/"

response = request("POST", URL + "display", data=buf)
print(response.text)
