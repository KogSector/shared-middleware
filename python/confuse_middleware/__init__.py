"""
ConFuse Shared Middleware - Python Package
Authentication, rate limiting, and security headers for FastAPI services.
"""
from confuse_middleware.auth import (
    AuthMiddleware,
    AuthenticatedUser,
    get_current_user,
    get_optional_user,
)
from confuse_middleware.rate_limit import RateLimitMiddleware
from confuse_middleware.security_headers import SecurityHeadersMiddleware

__all__ = [
    "AuthMiddleware",
    "AuthenticatedUser",
    "get_current_user",
    "get_optional_user",
    "RateLimitMiddleware",
    "SecurityHeadersMiddleware",
]
