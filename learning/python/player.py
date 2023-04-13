from game import Game, Status
from random import choices


class Player:
    opts: dict = {
        "history": [],
        "probabilities": {},
    }

    def play(game: Game, opts: dict):
        pass


class Human(Player):
    @staticmethod
    def play(game: Game) -> Game:
        while True:
            try:
                print(game.pretty())
                print(f"\n{game.next}'s Move: ", end="")
                move = int(input())
                n = sum(1 for i in range(move) if game[i] == " ")
                assert n < game.numMoves()
                game = game.move(move)
                Player.opts["history"].append((n, game))
                break
            except KeyboardInterrupt:
                print()
                exit()
            except ValueError:
                print("Invalid move!")

        return game


class Random(Player):
    @staticmethod
    def play(game: Game) -> Game:
        move, g = choices(list(enumerate(game)))[0]
        n = sum(1 for i in range(move) if game[i] == " ")
        assert n < game.numMoves()
        Player.opts["history"].append((n, g))

        return g


class AI(Player):
    @staticmethod
    def play(game: Game) -> Game:
        history = Player.opts["history"]
        probs = Player.opts["probabilities"]

        if str(game) not in probs:
            probs[str(game)] = [1 / game.numMoves()
                                for _ in range(game.numMoves())]

        move, g = choices(list(enumerate(game)), weights=probs[str(game)])[0]
        history.append((move, g))

        return g

    @staticmethod
    def adjust(result: Status, factor: float = 0.75):
        history = Player.opts["history"]
        probs = Player.opts["probabilities"]

        assert len(history) < 10

        if result == Status.won:
            last = len(history) - 1
            change = factor
        elif result == Status.lost:
            last = len(history) - 2
            change = -factor
        else:
            return

        for i in range(last, 0, -2):
            n, _ = history[i]
            _, g = history[i - 1]
            if str(g) not in probs:
                probs[str(g)] = [1 / g.numMoves() for _ in range(g.numMoves())]

            probs[str(g)][n] = probs[str(g)][n] + probs[str(g)][n] * \
                change if probs[str(g)][n] * change < 1 else 1
            change *= factor
