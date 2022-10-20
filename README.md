# eBook Cadeau

Service web pour générer des cartes cadeau pour accompagner des eBooks

## Getting Started

Pour travailler sur ce projet, il faut
[installer `composer`](https://getcomposer.org/doc/00-intro.md#installation-linux-unix-macos), et l'exécuter une fois:
```shell
composer install
```

On s'assure que le projet soit valide avec:
```shell
composer validate
```

### Exécution en local

À l'aide des outils CLI de Symfony:
```shell
symfony serve
```

### Déploiement local

Pour le déploiement du projet vers un serveur xampp local avec MINGW
```shell
cp -r * /c/xampp/htdocs/
```

### Déploiement distant

On utilise `rsync` pour déployer les fichiers vers le serveur SSH distant.
```shell
rsync --rsh='ssh -p 5022' -r --update --delete --progress ./ <serveur_de_preprod>:public_html
```

## Notes

- le `.htaccess` à la racine redirige le trafic vers le dossier `public/`