# ConFuse Common Python Library

Shared Python library for the ConFuse platform providing event schemas, middleware, and configuration utilities.

## Installation

```bash
pip install confuse-common
```

## Usage

### Events and Kafka
```python
from confuse_common import EventProducer, CodeIngestedEvent, Topics

# Create event producer
producer = EventProducer()

# Create and publish event
event = CodeIngestedEvent(
    headers=create_event_headers(),
    file_id="file123",
    file_path="/src/main.py",
    # ... other fields
)
await producer.publish(Topics.CODE_INGESTED, event)
```

### Authentication Middleware
```python
from confuse_common import AuthMiddleware, get_current_user
from fastapi import FastAPI

app = FastAPI()
app.add_middleware(AuthMiddleware)

# Get current user in endpoints
@app.get("/protected")
async def protected_route(current_user: AuthenticatedUser = Depends(get_current_user)):
    return {"message": f"Hello {current_user.user_id}"}
```

### Configuration
```python
from confuse_common import BaseConFuseApp, BaseConFuseSettings

class MyServiceSettings(BaseConFuseSettings):
    service_name: str = "my-service"
    port: int = Field(default=8000, alias="PORT")

class MyApp(BaseConFuseApp):
    def get_settings(self) -> BaseConFuseSettings:
        return MyServiceSettings()
    
    # ... implement other abstract methods
```

## Modules

- **confuse_common.events**: Event schemas and Kafka helpers
- **confuse_common.middleware**: Authentication, rate limiting, security headers
- **confuse_common.config**: Base classes and configuration utilities

## Version

0.2.0 - Breaking changes from previous package structure
