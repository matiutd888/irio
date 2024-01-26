from fastapi import FastAPI, HTTPException

app = FastAPI()
shutdown_requested = False

@app.get('/status', tags=['status'])
def status():
    if shutdown_requested:
        raise HTTPException(status_code=503, detail='Service is not available')
    return {'status': 'Success'}

@app.post('/shutdown', tags=['control'])
def shutdown():
    global shutdown_requested
    shutdown_requested = True
    return {'message': 'Server shutting down...'}

@app.post('/turnon', tags=['control'])
def turn_on():
    global shutdown_requested
    shutdown_requested = False
    return {'message': 'Server turned on'}

if __name__ == '__main__':
    import uvicorn
    uvicorn.run(app, host='0.0.0.0', port=8001)
