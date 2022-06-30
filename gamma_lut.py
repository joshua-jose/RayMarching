GAMMA = 2.2

lut = []
exp = 1/GAMMA

for i in range(256):
    corrected = round(255 * pow(i/255, exp))
    lut.append(corrected)

print(lut)
