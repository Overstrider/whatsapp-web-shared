# Smoke scripts

Run these end-to-end against a running backend (see `backend/README.md`).

```
./01_login.sh you@example.com
./02_qr.sh
./03_send.sh
```

All scripts store the session cookie in `/tmp/smoke.jar`.
