
# Systèmes numériques 2020

Un simulateur netlist écrit en [Rust](https://www.rust-lang.org/).

### Caractéristiques

Ce simulateur prend une netlist en entrée, la transforme en graphe
où les sommets sont des fils (ou des buses) et les arrêtes
sont des équations, puis opère un tri topologique pour obtenir
une liste d'instructions à exécuter pour faire simuler la netlist.

Lors de chaque tick, on exécute d'abord les opérations de mémoire "externe" :
lecture des entrées, des registres, des RAMs et des ROMs. On exécute
ensuite simplement chaque instruction à la suite.

Les données nécessaires à la simulation sont stockées dans un grand tableau :
on considère qu'un fil est un bus de taille 1, et on met tous les buses
côte à côte dans un tableau. À partir de ce moment, les buses ne sont
plus représentés que par leur adresse et leur longueur. Lors de la simulation,
toutes les opérations s'effectuent sur ce grand tableau.

La gestion de la RAM et de la ROM est similaire : on se donne une liste
de tableaux de booléens qui représentent la mémoire de chaque RAM ou ROM.

En l'état, ce simulateur ne permet pas d'initialiser les RAMs ni les ROMs,
mais il serait tout à fait possible de le faire (les ROMs sont effectivement
stockées en mémoire, mais sont systématiquements remplis de `false`s --
on aurait pu plus simplement ne pas allouer de mémoire pour les ROMs et
décréter que l'opération ROM renvoie toujours 0, ce qui aurait
été fonctionnellement équivalent).

### Compilation

 * Installer [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html);
 * Lancer `cargo build --release` (préciser `--release`, car le script `tests/tests.sh`
   suppose que le pogramme a été compilé en mode release);
 * L'exécutable est alors `../target/release/sim`.

### Exécution

Lorsque le projet est compilé, depuis ce dossier, lancer `../target/release/sim run test.net`.

### Tests

Le dossiers `tests` contient trois tests (opérations logiques et opérations sur les
bus au passage, RAM et registres).

Un test est représenté sous la forme de trois fichiers : une netlist (`test.net`),
une liste d'entrées (`test.in`, décomposée en sous-tests, séparés par des points-virgules),
et une liste de sorties (`test.out`, où les sous-tests sont aussi séparés par
des points-virgules).

Pour un lancer un test, on peut exécuter `../target/release/sim test tests/test`.
Le fichier `tests/tests.sh` permet de lancer les tests à la suite.

