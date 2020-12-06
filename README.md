
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

Le projet de simulateur pour Systèmes Numériques est complet. Il se trouve dans le dossier `sim`.

## Compilation

La phase 1 du projet de compilation est terminée !

Comme on veut rendre une archive "propre" pour le projet de Compilation, on se donne un petit
script (`compil/build_compil_bundle.sh`) qui génère les fichiers nécessaires.

