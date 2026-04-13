# Différences entre programmation embarquée et standard en Rust

La programmation embarquée en Rust diffère de la programmation standard sur plusieurs points:

- L'environnement d'exécution
- Le point d'entrer principale
- Dependence et structure.
- Compilation et cible
- Gestion de la mémoire

## Structure du projet

La structure d'un projet pour du *bare metal* n'est pas celle générée par `cargo new`. Le contenu est également différent.

## Objectif de cette étape

Cette étape vous permettra de découvrir :

- La structure d'un projet embarqué
- Les règles à respecter pour parvenir à compiler votre code sans erreur

# Résumé

La programmation standard en Rust s'appuie sur un OS et sa bibliothèque std, offrant confort et abstractions. La programmation embarquée (no_std) fonctionne sans OS, donne un contrôle matériel direct, mais nécessite de gérer manuellement les aspects bas niveau (allocateur, entrées/sorties, interruptions).

---

> **Pour plus d'informations, consultez le lien suivant :**  
> [https://esp32.implrust.com/std-to-no-std/index.html](https://esp32.implrust.com/std-to-no-std/index.html)
