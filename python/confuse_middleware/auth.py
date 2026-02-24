"""
Authentication middleware for FastAPI services.

Validates JWT tokens by calling the auth-middleware service.
Supports auth bypass for development mode.
"""
import os
from dataclasses import dataclass, field
from typing import Optional

import httpx
import structlog
from fastapi import Depends, HTTPException, Request, status
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer

logger = structlog.get_logger()
security = HTTPBearer(auto_error=False)


@dataclass
class AuthenticatedUser:
    """Authenticated user extracted from JWT or API key."""
    id: str
    email: str
    name: Optional[str] = None
    picture: Optional[str] = None
    roles: list[str] = field(default_factory=list)
    workspace_id: Optional[str] = None

    def has_role(self, role: str) -> bool:
        return role in self.roles


def _demo_user() -> AuthenticatedUser:
    """Demo user for auth bypass in development."""
    return AuthenticatedUser(
        id="demo-user-001",
        email="demo@confuse.dev",
        name="Demo User",
        roles=["user"],
        workspace_id="demo-workspace-001",
    )


class AuthMiddleware:
    """
    FastAPI dependency that validates authentication.

    Usage:
        auth = AuthMiddleware()

        @app.get("/protected")
        async def protected(user: AuthenticatedUser = Depends(auth.required)):
            return {"user": user.id}
    """

    def __init__(
        self,
        auth_service_url: Optional[str] = None,
        auth_bypass_enabled: Optional[bool] = None,
    ):
        self.auth_service_url = auth_service_url or os.getenv(
            "AUTH_MIDDLEWARE_URL", "http://auth-middleware:3010"
        )
        if auth_bypass_enabled is not None:
            self.auth_bypass_enabled = auth_bypass_enabled
        else:
            self.auth_bypass_enabled = os.getenv("AUTH_BYPASS_ENABLED", "false").lower() == "true"
        self._client = httpx.AsyncClient(timeout=5.0)

    async def _verify_token(self, token: str) -> AuthenticatedUser:
        """Validate a JWT token via auth-middleware."""
        try:
            resp = await self._client.post(
                f"{self.auth_service_url}/auth/validate",
                headers={"Authorization": f"Bearer {token}"},
            )
            if resp.status_code != 200:
                raise HTTPException(
                    status_code=status.HTTP_401_UNAUTHORIZED,
                    detail=f"Token validation failed: {resp.status_code}",
                )
            data = resp.json()
            return AuthenticatedUser(
                id=data.get("id", ""),
                email=data.get("email", ""),
                name=data.get("name"),
                picture=data.get("picture"),
                roles=data.get("roles", []),
                workspace_id=data.get("workspace_id"),
            )
        except httpx.RequestError as e:
            logger.error("Auth service unreachable", error=str(e))
            raise HTTPException(
                status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
                detail="Authentication service unavailable",
            )

    async def _verify_api_key(self, key: str) -> AuthenticatedUser:
        """Validate an API key via auth-middleware."""
        try:
            resp = await self._client.post(
                f"{self.auth_service_url}/auth/validate-api-key",
                headers={"X-API-Key": key},
            )
            if resp.status_code != 200:
                raise HTTPException(
                    status_code=status.HTTP_401_UNAUTHORIZED,
                    detail="Invalid API key",
                )
            data = resp.json()
            return AuthenticatedUser(
                id=data.get("user_id", data.get("id", "")),
                email=data.get("email", f"api-key@confuse.dev"),
                name=data.get("name"),
                roles=data.get("scopes", data.get("roles", [])),
                workspace_id=data.get("workspace_id"),
            )
        except httpx.RequestError as e:
            logger.error("Auth service unreachable", error=str(e))
            raise HTTPException(
                status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
                detail="Authentication service unavailable",
            )

    async def required(
        self,
        request: Request,
        credentials: Optional[HTTPAuthorizationCredentials] = Depends(security),
    ) -> AuthenticatedUser:
        """Dependency: require authentication (raises 401 if missing)."""
        if self.auth_bypass_enabled:
            return _demo_user()

        # Try Bearer token
        if credentials and credentials.credentials:
            user = await self._verify_token(credentials.credentials)
            # Check for workspace header
            ws_id = request.headers.get("x-workspace-id")
            if ws_id:
                user.workspace_id = ws_id
            return user

        # Try API key header
        api_key = request.headers.get("x-api-key")
        if api_key:
            user = await self._verify_api_key(api_key)
            ws_id = request.headers.get("x-workspace-id")
            if ws_id:
                user.workspace_id = ws_id
            return user

        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="No authentication provided",
        )

    async def optional(
        self,
        request: Request,
        credentials: Optional[HTTPAuthorizationCredentials] = Depends(security),
    ) -> Optional[AuthenticatedUser]:
        """Dependency: optional authentication (returns None if missing)."""
        if self.auth_bypass_enabled:
            return _demo_user()

        try:
            if credentials and credentials.credentials:
                user = await self._verify_token(credentials.credentials)
                ws_id = request.headers.get("x-workspace-id")
                if ws_id:
                    user.workspace_id = ws_id
                return user
        except HTTPException:
            pass

        return None


# Convenience instance
_default_auth = AuthMiddleware()


def get_current_user(
    request: Request,
    credentials: Optional[HTTPAuthorizationCredentials] = Depends(security),
) -> AuthenticatedUser:
    """Dependency shortcut for required auth."""
    import asyncio
    return asyncio.get_event_loop().run_until_complete(
        _default_auth.required(request, credentials)
    )


def get_optional_user(
    request: Request,
    credentials: Optional[HTTPAuthorizationCredentials] = Depends(security),
) -> Optional[AuthenticatedUser]:
    """Dependency shortcut for optional auth."""
    import asyncio
    return asyncio.get_event_loop().run_until_complete(
        _default_auth.optional(request, credentials)
    )
