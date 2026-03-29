from fastapi import FastAPI
from fastapi.responses import HTMLResponse
from pydantic import BaseModel

app = FastAPI()


class RepoSubmission(BaseModel):
    repo_url: str


@app.get("/", response_class=HTMLResponse)
async def index():
    with open("index.html") as f:
        return f.read()


@app.post("/submit")
async def submit_repo(submission: RepoSubmission):
    print(f"Repo submitted: {submission.repo_url}")
    return {"status": "received", "repo_url": submission.repo_url}
