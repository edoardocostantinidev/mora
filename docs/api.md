# API

Mora handles client requests via an HTTP API that exposes the various routes.
Here's a comprehensive list:

- `/health` 
  - `GET /`: returns `200 OK` if service is up and running correctly.
- `/queues`
  - `GET /`: gets all queues available
  - `GET /{queue_name}`: gets basic informations about a specific queue.
  - `POST /`: creates a queue.
  - `DELETE /{queue_name}`: deletes a queue by queue name.
- `/events`
  - `POST /`: schedules an event.
- `/channels`
  - `GET /`: retrieves all active channels
  - `GET /{channel_id}`: returns information about a specific channel
  - `GET /{channel_id}/events`: polling endpoint.
  - `POST /`: creates a new channel returning a unique ID and opening it.
  - `DELETE /{channel_id}`: closes a channel and deletes it.
  - `PUT /{channel_id}`: edits an active channel settings.

