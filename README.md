# IRIO alerting platform

Alerting platform that monitors a set of HTTP services.
When one of the services becomes unavailable the alerting platform sends a notification to the designated service administrator via Telegram.
In case the administrator does not respond the alerting platform sends a notification to the secondary service administrator.

## Deployment

The platform is deployed on GKE. It is distributed, highly available, scalable, and has a feature of autohealing. 
Every change in a codebase is automatically built and deployed using Github Actions.

## Testing

The platform has a testing cluster with a fake service deployed. The service is used in E2E tests that were implemented for the platform.
