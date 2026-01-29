#!/bin/bash

# Configuración fija
USER_NAME="rooselvelt6"
USER_EMAIL="rooselvelt6@gmail.com"
REPO_NAME="rocky"

echo "Iniciando subida al repositorio: $REPO_NAME"

# 1. Configurar identidad si no está lista
git config user.email "$USER_EMAIL"
git config user.name "$USER_NAME"

# 2. Inicializar y añadir archivos
if [ ! -d ".git" ]; then
    git init
    git remote add origin https://github.com/$USER_NAME/$REPO_NAME.git
    echo "Repositorio vinculado a GitHub."
fi

git add .

# 3. Commit con fecha y hora automática
commit_msg="Actualización automática: $(date +'%d-%m-%Y %H:%M:%S')"
git commit -m "$commit_msg"

# 4. Subir a la rama principal
git branch -M main

echo "------------------------------------------------------"
echo "IMPORTANTE: Cuando pida 'Password', PEGA TU TOKEN DE GITHUB"
echo "------------------------------------------------------"

git push -u origin main
