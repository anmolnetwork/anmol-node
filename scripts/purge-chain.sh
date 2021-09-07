#!/bin/bash
db=${1:-all}

if [[ "$OSTYPE" == "linux-gnu" ]]; then
  echo "Clearing local data from home dir: $HOME/.local/share/anmol"
	if [[ "$db" == "staging" ]]; then
		rm -rf ~/.local/share/anmol/chains/staging_testnet/db/
	elif [[ "$db" == "dev" ]]; then
		rm -rf ~/.local/share/anmol/chains/dev/db/
		rm -rf ~/.local/share/anmol/chains/development/db/
	elif [[ "$db" == "anmol" ]]; then
		rm -rf ~/.local/share/anmol/chains/anmol/db/
		rm -rf ~/.local/share/anmol/chains/ibtida/db/
	else
		rm -rf ~/.local/share/anmol/chains/dev/db/
		rm -rf ~/.local/share/anmol/chains/development/db/
		rm -rf ~/.local/share/anmol/chains/anmol/db/
		rm -rf ~/.local/share/anmol/chains/ibtida/db/ # Anmol's Testnet database
		rm -rf ~/.local/share/anmol/chains/staging_testnet/db/
		rm -rf ~/.local/share/anmol/chains/local_testnet/db/
        rm -rf ~/.local/share/anmol/chains/$db/db/
	fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Clearing local data from home dir: $HOME/Library/Application Support/anmol"
	if [[ "$db" == "staging" ]]; then
		rm -rf ~/Library/Application\ Support/anmol/chains/staging_testnet/db/
	elif [[ "$db" == "dev" ]]; then
		rm -rf ~/Library/Application\ Support/anmol/chains/dev/db/
		rm -rf ~/Library/Application\ Support/anmol/chains/development/db/
	elif [[ "$db" == "anmol" ]]; then
		rm -rf ~/Library/Application\ Support/anmol/chains/anmol/db/
		rm -rf ~/Library/Application\ Support/anmol/chains/ibtida/db/ 
	else
		rm -rf ~/Library/Application\ Support/anmol/chains/dev/db/
		rm -rf ~/Library/Application\ Support/anmol/chains/development/db/
		rm -rf ~/Library/Application\ Support/anmol/chains/anmol/db/
		rm -rf ~/Library/Application\ Support/anmol/chains/ibtida/db/
		rm -rf ~/Library/Application\ Support/anmol/chains/staging_testnet/db/
		rm -rf ~/Library/Application\ Support/anmol/chains/local_testnet/db/
        rm -rf ~/Library/Application\ Support/anmol/chains/$db/db/
	fi
else
  echo "Clearing local data from home dir: $HOME/.local/share/anmol"
	if [[ "$db" == "staging" ]]; then
		rm -rf ~/.local/share/anmol/chains/staging_testnet/db/
	elif [[ "$db" == "dev" ]]; then
		rm -rf ~/.local/share/anmol/chains/dev/db/
		rm -rf ~/.local/share/anmol/chains/development/db/
	elif [[ "$db" == "anmol" ]]; then
		rm -rf ~/.local/share/anmol/chains/anmol/db/
		rm -rf ~/.local/share/anmol/chains/ibtida/db/
	else
		rm -rf ~/.local/share/anmol/chains/dev/db/
		rm -rf ~/.local/share/anmol/chains/development/db/
		rm -rf ~/.local/share/anmol/chains/anmol/db/
		rm -rf ~/.local/share/anmol/chains/ibtida/db/
		rm -rf ~/.local/share/anmol/chains/staging_testnet/db/
		rm -rf ~/.local/share/anmol/chains/local_testnet/db/
        rm -rf ~/.local/share/anmol/chains/$db/db/
	fi
fi

echo "Deleted $db databases"