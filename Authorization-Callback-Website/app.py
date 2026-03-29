import os
import sys
from pathlib import Path

from fastapi import FastAPI
from fastapi.responses import HTMLResponse
from pydantic import BaseModel

sys.path.append(str(Path(__file__).parent.parent))
from chunker import main as chunk_repo
from github_auth import get_token_for_repo

app = FastAPI()

APP_ID = os.environ["GITHUB_APP_ID"]
PRIVATE_KEY = Path(os.environ["GITHUB_PRIVATE_KEY_PATH"]).read_text()


class RepoSubmission(BaseModel):
    repo_url: str


@app.get("/", response_class=HTMLResponse)
async def index():
    with open("index.html") as f:
        return f.read()


@app.post("/submit")
async def submit_repo(submission: RepoSubmission):
    repo_owner = submission.repo_url.split("/")[3]
    token = get_token_for_repo(APP_ID, PRIVATE_KEY, repo_owner)
    await chunk_repo(submission.repo_url, token=token)
    return {"status": "done", "repo_url": submission.repo_url}
