#!/bin/bash
# Doppelklick startet die CDBrain Slide-Player-Verwaltung lokal.
cd "$(dirname "$0")"
exec python3 server.py
