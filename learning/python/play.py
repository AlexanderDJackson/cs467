from player import AI, Human, Random, Player, Status
from game import Game, TicTacToe
import json


def main():
    game = TicTacToe("         ")

    try:
        with open("probs.json", "r") as f:
            Player.opts["probabilities"] = json.load(f)
    except FileNotFoundError:
        pass

    draws = 0
    games = 1000

    for i in range(games):
        if play(game, player1=AI, player2=AI) == Status.draw:
            draws += 1

    print(f"Draw rate = {(draws / games):.2%}")

    with open("probs.json", "w") as f:
        json.dump(Player.opts["probabilities"], f, indent=2)


def play(game: Game, player1: Player = Human, player2: Player = AI):
    while True:
        game = player1.play(game)

        if game.getStatus() == Status.won:
            if player1 == AI:
                player1.adjust(Status.won)

            if player2 == AI:
                player2.adjust(Status.lost)

            break
        elif game.getStatus() == Status.draw:
            break

        game = player2.play(game)

        if game.getStatus() == Status.won:
            if player1 == AI:
                player1.adjust(Status.lost)

            if player2 == AI:
                player2.adjust(Status.won)

            break
        elif game.getStatus() == Status.draw:
            break

    if player1 == Human or player2 == Human:
        print(game.pretty())
        print("Game over: ", end="")
        if game.getStatus() == Status.draw:
            print("Draw")
        else:
            print(f"{game.winner} won")

    Player.opts["history"].clear()
    return game.getStatus()


if __name__ == "__main__":
    main()
