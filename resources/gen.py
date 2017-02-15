from PIL import Image, ImageDraw, ImageFont

font = ImageFont.truetype("/usr/share/fonts/noto/NotoMono-Regular.ttf", 50)

list(map(str,range(2, 11)))+["jack","queen","king"]
for suit in ["spades", "hearts", "clubs", "diamonds"]:
    color = "black" if suit == "spades" or suit == "clubs" else "red"
    base = Image.open("cards/card_ace_{}.png".format(suit))
    for card in list(map(str,range(2, 11)))+["jack","queen","king"]:
        im = base.copy()#Image.new("RGBA", (162, 252), "white")
        draw = ImageDraw.Draw(im)
        draw.text((0,0), "{}".format(card), color, font)
        del draw
        im.save("cards/card_{}_{}.png".format(card, suit), "PNG", dpi=(72,72))
