# /bin/bash

ansible-galaxy install --force -r playbooks/requirements.yml
sudo ansible-playbook playbooks/minecraft-server.yml
