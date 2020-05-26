# doesn't really work or is extremely slow at this moment
from bitarray import bitarray
import random

def gen_boards():
    result = list()
    gen_boards_helper('', result, 0)
    return result


def gen_boards_helper(cur, result, bits):
    if bits == 9:
        result.append(cur)
        return
    
    gen_boards_helper(cur + '-', result, bits + 1)
    gen_boards_helper(cur + 'X', result, bits + 1)
    gen_boards_helper(cur + 'O', result, bits + 1)


def to_bits(board):
    ret = 0
    for i, c in enumerate(board):
        if c == 'X':
            ret |= 1 << i
        elif c == 'O':
            # stacked on top of X board
            ret |= 1 << (i + 9)
        else:
            assert c == '-'
    return ret

boards = gen_boards()
boards = [to_bits(x) for x in boards]
b17 = 0b11111111111111111
# generates: 536887361 in 1!! loop. to hash from 18-bit two-board do
# (SOURCE * MAGIC) % 262144
l = 0
count = 0
nbits = 17
length = 2 ** nbits
while True:
    l += 1
    mem = bitarray(length)
    n = random.randint(0, 2147483647) & random.randint(0, 2147483647) & random.randint(0, 2147483647)
    good = True
    for b in boards:
        target = (b * n) & b16
        if mem[target] == True:
            good = False
            break
        mem[target] = True
    if good: 
        print(n)
        break


