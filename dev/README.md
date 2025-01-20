Here are some useful development things;

```bash
cd dev/
DEBUG=true ./jobd &
curl -X POST -F "file=@input.zip" -F "data=@data.json" http://127.0.0.1:3000/upload
```
