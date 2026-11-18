FROM python:3.11-slim

LABEL org.opencontainers.image.source="https://github.com/MacroMag-os/MacroMag-32"
LABEL org.opencontainers.image.description="MacroMag-32 Runtime & Virtualized Object Shell (VOS) Prototype"
LABEL org.opencontainers.image.licenses="GPL-3.0"

WORKDIR /macromag
COPY . /macromag

CMD ["python3", "powershell32.py"]
