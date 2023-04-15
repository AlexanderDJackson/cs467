from bitarray import bitarray
from curses import wrapper


class Board:
    board: bitarray
    mask: bitarray
    next: bool

    def __init__(self, board: bitarray, mask: bitarray, next: bool = True):
        self.board = board
        self.mask = mask
        self.next = next

    @staticmethod
    def from_string(board: str, next: str):
        b = bitarray(board
                     .replace("X", "1")
                     .replace("O", "0")
                     .replace("-", "0"))
        m = bitarray(board
                     .replace("X", "1")
                     .replace("O", "1")
                     .replace("-", "0"))

        return Board(b, m, next == "X")

    def check_directions(self, x: int) -> int:
        directions = bitarray('0' * 8)

        # Check clockwise:
        # top left, top, top right,
        # right, bottom right,
        # bottom, bottom left, left
        for n, offset in enumerate([-7, -6, -5, 1, 7, 6, 5, -1]):
            to_flip = False
            old = x
            for i in range(x + offset, -1 if offset < 0 else 36, offset):
                if -1 <= (old % 6) - (i % 6) <= 1:  # bounds check
                    if not self.mask[i]:  # If the space is empty
                        break
                    if (self.board[i] & self.mask[i]) ^ self.next:
                        to_flip = True
                    elif to_flip:
                        directions[n] = 1
                        break
                    else:
                        break
                    old = i
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

        return f"{b[:6]}\n" \
            f"{b[6:12]}\n" \
            f"{b[12:18]}\n" \
            f"{b[18:24]}\n" \
            f"{b[24:30]}\n" \
            f"{b[30:]}"

    def move(self, x: int, directions: int):
        new_mask = bitarray(self.mask)
        new_mask[x] = 1

        new_board = bitarray(self.board)
        new_board[x] = self.next

        for n, offset in enumerate([-7, -6, -5, 1, 7, 6, 5, -1]):
            if directions[n]:
                for i in range(x + offset, -1 if offset < 0 else 36, offset):
                    if (self.board[i] & self.mask[i]) ^ self.next:
                        new_board[i] = self.next
                        new_mask[i] = 1
                    else:
                        break

        return Board(new_board, new_mask, not self.next)

    def moves(self):
        moves = []

        for i in range(36):
            if not self.mask[i]:
                valid = self.check_directions(i)
                if valid.any():
                    moves.append(self.move(i, valid))

        return moves

    def pretty(self, color: bool = False) -> str:
        if color:
            # Print the background in gray
            # Make the X's Black and the O's White
            def color(s: str):
                return f"\033[48;5;8m\033[38;5;0m{s}\033[0m"
        else:
            def color(s: str):
                return s

        b = "╔═══╦═══╦═══╦═══╦═══╦═══╗\n║"
        for i in range(36):
            if i % 6 == 0 and i != 0:
                b += "\n╠═══╬═══╬═══╬═══╬═══╬═══╣\n║"

            if self.mask[i]:
                b += f" {'X' if self.board[i] else 'O'} ║"
            else:
                b += "   ║"
        b += "\n╚═══╩═══╩═══╩═══╩═══╩═══╝"
        return color(b)


def main(stdscr):
    stdscr.clear()

    _ = Board(
        bitarray('000000000000001000000100000000000000'),
        bitarray('000000000000001100001100000000000000'),
        True
    )
    board = Board.from_string("--------------XO----OX--------------", "X")

    stdscr.addstr(board.pretty())


if __name__ == "__main__":
    wrapper(main)
