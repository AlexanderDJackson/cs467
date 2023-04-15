from bitarray import bitarray


class Board:
    def __init__(self, board="--------------XO----OX--------------", next="X"):
        if type(board) == tuple and type(board[0]) == bitarray and type(board[1]) == bitarray:
            self.board = board[0]
            self.mask = board[1]
            if type(next) == bool:
                self.next = next
            else:
                if type(next) == str:
                    self.next = 1 if next == "X" else 0
                else:
                    raise TypeError(f"Expected next to be bool, since board is a tuple, but got type {type(next)}")
        elif type(board) == str:
            self.board = bitarray(board.replace(
                "-", "0").replace("X", "1").replace("O", "0"))
            self.mask = bitarray(board.replace(
                "-", "0").replace("X", "1").replace("O", "1"))
            if type(next) == str:
                self.next = 1 if next == "X" else 0
            else:
                if type(next) == bool:
                    self.next = next
                else:
                    raise TypeError(f"Expected next to be str, since board is a str, but got type {type(next)}")

    def check_directions(self, x: int) -> int:
        directions = bitarray(8)

        # Check clockwise: top left, top, top right, right, bottom right, bottom, bottom left, left
        for n, offset in enumerate([-7, -6, -5, 1, 7, 6, 5, -1]):
            to_flip = False
            for i in range(x + offset, -1 if offset < 0 else 36, offset):
                if not (self.board[i] or self.mask[i]):  # If the space is empty
                    break
                if not ((self.board[i] & self.mask[i]) ^ self.next):
                    to_flip = True
                elif to_flip:
                    directions[n] = 1
                    break
                else:
                    break
        return directions

    def __str__(self) -> str:
        b = []
        for i in self.mask:
            b.append("X" if i else "-")

        for i in range(36):
            b[i] = "O" if self.board[i] ^ self.mask[i] else b[i]

        b = "".join(b)

        return f"{b[:6]}\n{b[6:12]}\n{b[12:18]}\n{b[18:24]}\n{b[24:30]}\n{b[30:]}"

    def move(self, x: int, directions: int):
        new_mask = bitarray(self.mask)
        new_mask[x] = 1

        new_board = bitarray(self.board)
        new_board[x] = self.next

        for n, offset in enumerate([-7, -6, -5, 1, 7, 6, 5, -1]):
            if directions[n]:
                for i in range(x + offset, -1 if offset < 0 else 36, offset):
                    if not ((self.board[i] & self.mask[i]) ^ self.next):
                        new_board[i] = self.next
                    else:
                        break

        print(x)
        print(new_board)
        print(new_mask)
        return Board((new_board, self.mask), not self.next)

    def moves(self):
        moves = []

        for i in range(36):
            if not self.mask[i]:
                valid = self.check_directions(i)
                if valid:
                    moves.append(self.move(i, valid))

        return moves


def main():
    # board = Board("--------------XO----OX--------------", "X")
    board = Board("OXXXXX-XXXXX--XO----OX--------------", "O")
    print(f"Next: {board.next}\n{board}\n")

    for move in board.moves():
        pass
        # print(f"Next: {move.next}\n{move}\n")


if __name__ == "__main__":
    main()
