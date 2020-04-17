"""
generate win table for 3x3 blocks and output hex string.
the bit at position i if the position denoted
by the bit-level representation of i is won.
For example 0b000000111 corresponds to:
X X X
_ _ _
_ _ _
(only the pieces placed by one side is included)
"""

ROWS = [0b111, 0b111000, 0b111000000]
COLS = [0b001001001, 0b010010010, 0b100100100]
DIAGS = [0b001010100, 0b100010001]

res = [0 for _ in range(512)]

def compute_result(occ):
    for row in ROWS:
        if row & occ == row:
            return 1

    for col in COLS:
        if col & occ == col:
            return 1

    for diag in DIAGS:
        if diag & occ == diag:
            return 1

    return 0

def gen_win(cur, count):
    if count == 0:
        print("{0:b}".format(cur), compute_result(cur))
        res[cur] = compute_result(cur)
    else:
        gen_win(cur << 1, count - 1)
        gen_win((cur << 1) + 1, count - 1)


gen_win(0, 9)
b2 = ''.join([str(x) for x in reversed(res)])
for i in reversed(range(8)):
    print(hex(int(b2[i * 64:i * 64 + 64], 2)) + ',')
