class Board:
    def __init__(self, board = "--------------XO----OX--------------", next = "X"):
        self.board = board
        self.next = next

    def check_directions(self, x: int) -> int:
        directions = 0

        # Check clockwise: top left, top, top right, right, bottom right, bottom, bottom left, left
        for offset in (-7, -6, -5, 1, 7, 6, 5, -1):
            to_flip = False
            for i in range(x + offset, -1 if offset < 0 else 36, offset): 
                if self.board[i] == "-":
                    break
                elif self.board[i] != self.next:
                    to_flip = True
                elif to_flip:
                    directions |= 1
                    break
                else:
                    break
            directions <<= 1
            print(f"Directions: {bin(directions)}")
        return directions >> 1

    def __str__(self) -> str:
        b = self.board
        return f"{b[:6]}\n{b[6:12]}\n{b[12:18]}\n{b[18:24]}\n{b[24:30]}\n{b[30:]}"

    def move(self, x: int, directions: int):
        new_board = list(self.board)
        new_board[x] = self.next

        if directions == 0:
            self

        print(f"Directions: {bin(directions)}")
        for offset in (-7, -6, -5, 1, 7, 6, 5, -1):
            if directions & 0b10000000:
                for i in range(x + offset, -1 if offset < 0 else 36, offset): 
                    if self.board[i] == self.next:
                        break
                    elif self.board[i] != "-":
                        print(f"Writing {self.next} to {i}")
                        new_board[i] = self.next
            directions <<= 1

        return Board("".join(new_board), "X" if self.next == "O" else "O")

    """
    Reversi Board Format:
        --------------XO----OX--------------

    Legal Moves (marked with a "*"):
        ------
        ---*--
        --XO*-
        -*OX--
        --*---
        ------
    """
    def moves(self):
        moves = []

        for i in range(36):
            valid = self.check_directions(i)
            if self.board[i] == "-" and valid != 0:
                moves.append(self.move(i, valid))

        return moves


def count(board: Board) -> int:
    m = board.moves()

    if len(m) == 0:
        return 1
    else:
        return sum(count(move) for move in m)

def main():
    # board = Board("--------------XO----OX--------------", "X")
    board = Board("OXXXXX-XXXXX--XO----OX--------------", "O")
    print(f"Next: {board.next}\n{board}\n")
    board = board.move(6, board.check_directions(6))
    print(f"Next: {board.next}\n{board}\n")

    # for move in moves(board):
        # print(count(move))

if __name__ == "__main__":
    main()
