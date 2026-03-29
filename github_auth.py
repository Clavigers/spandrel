import time
import httpx
import jwt

GITHUB_API = "https://api.github.com"


def _generate_jwt(app_id: str, private_key: str) -> str:
    now = int(time.time())
    payload = {
        "iat": now - 60,   # 60s leeway for clock drift
        "exp": now + 600,  # 10 minute max
        "iss": app_id,
    }
    return jwt.encode(payload, private_key, algorithm="RS256")


def _jwt_headers(app_id: str, private_key: str) -> dict:
    return {
        "Authorization": f"Bearer {_generate_jwt(app_id, private_key)}",
        "Accept": "application/vnd.github+json",
    }


def list_installations(app_id: str, private_key: str) -> list[dict]:
    response = httpx.get(
        f"{GITHUB_API}/app/installations",
        headers=_jwt_headers(app_id, private_key),
    )
    response.raise_for_status()
    return response.json()


def get_installation_token(app_id: str, private_key: str, installation_id: int) -> str:
    response = httpx.post(
        f"{GITHUB_API}/app/installations/{installation_id}/access_tokens",
        headers=_jwt_headers(app_id, private_key),
    )
    response.raise_for_status()
    return response.json()["token"]


def get_token_for_repo(app_id: str, private_key: str, repo_owner: str) -> str:
    """Find the installation for repo_owner and return an access token."""
    installations = list_installations(app_id, private_key)
    for installation in installations:
        if installation["account"]["login"].lower() == repo_owner.lower():
            return get_installation_token(app_id, private_key, installation["id"])
    raise ValueError(f"No installation found for owner '{repo_owner}'")
