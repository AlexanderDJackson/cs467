#!/bin/python

import argparse
import random
import string

def get_ids(n):
    alphabet = string.ascii_uppercase
    result = []
    for i in range(n):
        if i < 26:
            result.append(alphabet[i])
        else:
            result.append(alphabet[i // 26 - 1] + alphabet[i % 26])
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description = "A utility to generate knapsack data sets")
    parser.add_argument("-n", "--number", required = True, type = int)
    parser.add_argument("-l", "--weight-limit", required = True, type = int)
    parser.add_argument("-W", "--max-weight", default = 10, type = int)
    parser.add_argument("-w", "--min-weight", default = 1, type = int)
    parser.add_argument("-V", "--max-value", default = 10, type = int)
    parser.add_argument("-v", "--min-value", default = 1, type = int)
    parser.add_argument("-d", "--distinct-weights", action = "store_true")
    parser.add_argument("-D", "--distinct-values", action = "store_true")

    args = vars(parser.parse_args())

    number = args["number"]
    weight_limit = args["weight_limit"]
    max_weight = args["max_weight"]
    min_weight = args["min_weight"]
    max_value = args["max_value"]
    min_value = args["min_value"]
    distinct_weights = args["distinct_weights"]
    distinct_values = args["distinct_values"]

    output = str(number) + ", " + str(weight_limit)

    #Get the list of item IDs
    ids = get_ids(number)

    #Create a pool of all weights for random choice
    if args["distinct_weights"]:
        weights = [x for x in range(min_weight, max_weight + 1)]
    else:
        weights = [random.randrange(min_weight, max_weight + 1, 1) for x in range(number)]

    #Create a pool of all values for random choice
    if args["distinct_values"]:
        values = [x for x in range(min_value, max_value + 1)]
    else:
        values = [random.randrange(min_value, max_value + 1, 1) for x in range(number)]

    pairs = [(weights[i], values[i]) for i in range(0, number)]

    for i in range(number):
        output += "\n" + str(ids[i]) + "," + str(weights[i]) + "," + str(values[i])

    print(output)

