#!/bin/python
import sys
import subprocess
from tqdm import tqdm
import json

files = sys.argv[1:]
intermediate_sizes = [0.5, 1, 2]
population_sizes = [100]
generation_sizes = [100, 1000]
sex_methods = ["one", "two", "uniform"]
selection_methods = ["equal", "replacement", "remainder"]
crossover_rates = [0.1, 0.25, 0.50]
mutation_rates = [0.01, 0.03, 0.05, 0.1]

def get_results(generation_size, population, intermediate, sex_method, selection_method, crossover_rate, mutation_rate):
    proc = subprocess.Popen(['genetic', '-r', 'stocks', '--file'] + files + ['-p', str(population), '-i', str(int(population * intermediate)), '-x', str(sex_method), '-s', str(selection_method), '-k', str(crossover_rate), '-m', str(mutation_rate), '-M', str(generation_size)], stdout=subprocess.PIPE)
    return proc.stdout.read().decode('utf-8').strip()

with open('results.json', 'w') as f:
    with open('tmp.txt', 'a') as t:
        with tqdm(total = len(intermediate_sizes) * len(generation_sizes) * len(population_sizes) * len(sex_methods) * len(selection_methods) * len(crossover_rates) * len(mutation_rates)) as pb:
            results = {}
            for generation_size in generation_sizes:
                for population_size in population_sizes:
                    for intermediate_size in intermediate_sizes:
                        for sex_method in sex_methods:
                            for selection_method in selection_methods:
                                for crossover_rate in crossover_rates:
                                    for mutation_rate in mutation_rates:
                                        t.write(f"\n\"{generation_size},{population_size},{int(population_size * intermediate_size)},{sex_method},{selection_method},{crossover_rate},{mutation_rate}\"\n")
                                        tmp = []
                                        for i in range(0,10):
                                            tmp.append(get_results(generation_size, population_size, intermediate_size, sex_method, selection_method, crossover_rate, mutation_rate))
                                        results[f"{generation_size},{population_size},{int(population_size * intermediate_size)},{sex_method},{selection_method},{crossover_rate},{mutation_rate}"] = tmp
                                        pb.update(1)
                                        t.write(json.dumps(tmp, indent = 4))
            f.write(json.dumps(results, indent = 4))

