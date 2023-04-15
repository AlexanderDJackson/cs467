from main import Game, pretty_print
from randomplayer import RandomPlayer

# decide if pieces are flippable in this direction


def flips(board, index, piece, step):
    other = ('X' if piece == 'O' else 'O')
    # is an opponent's piece in first spot that way?
    here = index + step
    if board[here] != other:
        return False
    # diff : how index mod is changing each step
    diff = index % 6 - here % 6
    while here % 6 - (here + step) % 6 == diff and \
            here > 0 and here < 36 and \
            board[here] == other:
        here = here + step

    return here % 6 - (here + step) % 6 == diff and \
        here > 0 and here < 36 and \
        board[here] == piece

# decide if given move (index x) is valid for player p


def validMove(b, x, p):  # board, index, piece
    # invalid index
    if x < 0 or x >= 36:
        return False
    # space already occupied
    if b[x] != '-':
        return False
    # otherwise, check for flipping pieces
    up = x >= 12   # at least third row down
    down = x < 24   # at least third row up
    left = x % 6 > 1  # at least third column
    right = x % 6 < 4  # not past fourth column
    return (left and flips(b, x, p, -1)  # left
            or up and left and flips(b, x, p, -7)  # up/left
            or up and flips(b, x, p, -6)  # up
            or up and right and flips(b, x, p, -5)  # up/right
            or right and flips(b, x, p, 1)  # right
            or down and right and flips(b, x, p, 7)  # down/right
            or down and flips(b, x, p, 6)  # down
            or down and left and flips(b, x, p, 5))  # down/left

# actually flip pieces in this direction
# assume validity has already been checked


def applyFlip(board, index, piece, step):
    other = ('X' if piece == 'O' else 'O')
    # starting point
    here = index + step
    while board[here] == other:
        board = board[:here] + piece + board[here+1:]
        here = here + step
    return board

# actually flip pieces in this direction


def applyMove(x, p):  # index, piece
    global gameboard
    b = gameboard

    # if not valid move, stop here
    if not validMove(b, x, p):
        return False

    up = x >= 12   # at least third row down
    down = x < 24   # at least third row up
    left = x % 6 > 1  # at least third column
    right = x % 6 < 4  # not past fourth column

    # flip everything that should be flipped
    if left and flips(b, x, p, -1):  # left
        b = applyFlip(b, x, p, -1)
    if up and left and flips(b, x, p, -7):  # up/left
        b = applyFlip(b, x, p, -7)
    if up and flips(b, x, p, -6):  # up
        b = applyFlip(b, x, p, -6)
    if up and right and flips(b, x, p, -5):  # up/right
        b = applyFlip(b, x, p, -5)
    if right and flips(b, x, p, 1):  # right
        b = applyFlip(b, x, p, 1)
    if down and right and flips(b, x, p, 7):  # down/right
        b = applyFlip(b, x, p, 7)
    if down and flips(b, x, p, 6):  # down
        b = applyFlip(b, x, p, 6)
    if down and left and flips(b, x, p, 5):  # down/left
        b = applyFlip(b, x, p, 5)
    # and put a new piece here too
    b = b[:x] + p + b[x+1:]
    # save modified gameboard
    gameboard = b


def printBoard(board):
    print()
    print("##########")
    print("# " + board[0: 6] + " #")
    print("# " + board[6:12] + " #")
    print("# " + board[12:18] + " #")
    print("# " + board[18:24] + " #")
    print("# " + board[24:30] + " #")
    print("# " + board[30:36] + " #")
    print("##########")
    print()

# how many moves does this player have currently available?


def countPossibleMoves(board, piece):
    movesLeft = 0
    for i in range(36):
        movesLeft = movesLeft + validMove(board, i, piece)
    return movesLeft

# game score given board layout
# X wins if positive, O wins if negative, tie if zero


def getEndgameStatus(board):

    countX = 0
    countO = 0
    for i in range(36):
        countX = countX + (board[i] == 'X')
        countO = countO + (board[i] == 'O')

    return countX - countO


# global variables
gameboard = "--------------XO----OX--------------"
gameover = False

playerX = RandomPlayer('X')
playerO = RandomPlayer('O')
# O = Agent('O') # use this when agent is implemented

# counters for tracking wins over multiple trials
numWinX = 0
numWinO = 0
numTied = 0

moves = []


def log(moves):
    global gameboard
    gameboard = "--------------XO----OX--------------"
    with open('given.txt', 'w') as g:
        with open('mine.txt', 'w') as m:
            g.write(f"X's Turn:\n{pretty_print(gameboard)}\n")
            m.write(f"X's Turn:\n{pretty_print(str(Game.from_string(gameboard, 'X')))}\n")
            player = 'O'
            for move in moves:
                print(gameboard)
                board = Game.from_string(gameboard, 'X')
                applyMove(move, player)
                g.write(f"{player} played {move}:\n{pretty_print(gameboard)}\n")
                m.write(
                    f"{player} played {move}:\n{pretty_print(str(board.move(move, board.check_directions(move))))}\n")
                player = 'X' if player == 'O' else 'O'

# how many games do you want to play?
for g in range(2500):
    # reset global variables for new game
    gameboard = "--------------XO----OX--------------"
    gameover = False

    g = open('given.txt', 'w')
    m = open('mine.txt', 'w')
    
    # play game until done
    move = 1
    while not gameover:
        # assert countPossibleMoves(gameboard, 'O') == \
        # len(Game.from_string(gameboard, 'O').moves())

        if countPossibleMoves(gameboard, 'X') > 0:
            play = -1
            while not validMove(gameboard, play, 'X'):
                play = playerX.getMove(gameboard)
            temp = Game.from_string(gameboard, 'X')
            applyMove(play, 'X')
            directions = temp.check_directions(play)
            g.write(f"O played {play}:\n{pretty_print(gameboard)}\n")
            m.write(
                f"O played {play}:\n{pretty_print(str(temp.move(play, directions)))}\n")
            try:
                assert gameboard == str(temp.move(play, directions))
            except AssertionError:
                g.close()
                m.close()
                exit()

        # player O
        if countPossibleMoves(gameboard, 'O') > 0:
            play = -1
            while not validMove(gameboard, play, 'O'):
                play = playerO.getMove(gameboard)
            temp = Game.from_string(gameboard, 'O')
            applyMove(play, 'O')
            directions = temp.check_directions(play)
            g.write(f"O played {play}:\n{pretty_print(gameboard)}\n")
            m.write(
                f"O played {play}:\n{pretty_print(str(temp.move(play, directions)))}\n")
            try:
                assert gameboard == str(temp.move(play, directions))
            except AssertionError:
                g.close()
                m.close()
                exit()

        # if game over
        if countPossibleMoves(gameboard, 'X') + \
                countPossibleMoves(gameboard, 'O') == 0:
            status = getEndgameStatus(gameboard)
            if status > 0:  # X wins
                playerX.endGame(1, gameboard)
                playerO.endGame(-1, gameboard)
                numWinX = numWinX + 1
                # print( "X wins by " + str(status) + " pieces" )
            elif status < 0:  # O wins
                playerX.endGame(-1, gameboard)
                playerO.endGame(1, gameboard)
                numWinO = numWinO + 1
                # print( "O wins by " + str(-status) + " pieces" )
            else:  # status == 0, tie game
                playerX.endGame(0, gameboard)
                playerO.endGame(0, gameboard)
                numTied = numTied + 1
                # print( "Tie game" )
            gameover = True
            # printBoard(gameboard)

        move = move + 1

    # when running thousands of learning trials,
    #   periodic updates are nice confirmation
    #   that everything's still running
#   if (numWinX + numWinO + numTied) % 1000 == 0:
#      print( "Completed " + str(numWinX + numWinO + numTied) )

playerX.stopPlaying()
playerO.stopPlaying()

print("X   : " + str(numWinX) + " games")
print("O   : " + str(numWinO) + " games ***")
print("Tie : " + str(numTied) + " games")
