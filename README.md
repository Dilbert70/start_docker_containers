Startar alla docker containrar som fins i undermappar i form av docker compose. Förutom dom conatinrar som har en fil som heter no_autostart, dom containrarna förblir stoppade.
Skapa no_autostart filen enklast med "touch no_autostart"

Starts all docker containers in subfolders if they are docker compose. To prevent start of a container put a file named no_autostart in the same folder as the docker-compose file, the file can bee empty.
