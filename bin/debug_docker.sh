#!/bin/bash
echo "=== DEBUG START ===" > debug_output.txt
date >> debug_output.txt
echo "--- Docker PS ---" >> debug_output.txt
docker ps -a >> debug_output.txt 2>&1
echo "--- Compose Up ---" >> debug_output.txt
docker-compose up -d --force-recreate >> debug_output.txt 2>&1
echo "--- Logs SurrealDB ---" >> debug_output.txt
sleep 5
docker logs rocky-surrealdb >> debug_output.txt 2>&1
echo "--- Final State ---" >> debug_output.txt
docker ps -a >> debug_output.txt 2>&1
echo "=== DEBUG END ===" >> debug_output.txt
