from PIL import Image, ImageOps

from requests import request

URL = "http://192.168.1.139/"

response = request("GET", URL)
print(response.text)


WIDTH = 800
HEIGHT = 480

image = Image.open('image.png')

if image.size[0] == WIDTH and image.size[1] == HEIGHT:
    image = image.rotate(90, expand=True)

elif image.size[0] != WIDTH or image.size[1] != HEIGHT:
    image = ImageOps.pad(image=image, size=(WIDTH, HEIGHT), color="white")

buf = bytearray(image.convert('1').tobytes('raw'))

# The bytes need to be inverted, because in the PIL world 0=black and 1=white, but
# in the e-paper world 0=white and 1=black.
# for i in range(len(buf)):
#     buf[i] ^= 0xFF

# data = ("a" * 48_000)


response = request("POST", URL + "display", data=buf)
print("Text:", response.text)
