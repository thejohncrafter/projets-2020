
# Projets d'info 2020

Aussi sur [Github](https://github.com/thejohncrafter/projets-2020) !

Une collection de projets d'info pour 2020.

Certains composants sont réutilisés entre les projets (pour l'instant `automata` et `parsergen`).

###### N.B.
Dans un futur plus ou moins proche, les projets seront découplés et auront chacun leur dépôt, mais en attendant tout réside ici.

## Générateur de parseurs

Ce générateur est contenu dans les dossiers `automata` et `parsergen`.
C'est un générateur de parseur LR(1) qui exploite grandement le système
de [macros procédurales](https://doc.rust-lang.org/reference/procedural-macros.html) de Rust.

Un lexeur et un parseur sont implémentés avec ce système dans le fichier `sim/src/parsing/parser.rs`

## Systèmes numériques

En l'état actuel, seul le projet de simulateur pour Systèmes Numériques est complet. Il se trouve dans le dossier `sim`.

