# API

Mora handles client requests via an HTTP API that exposes the various routes.
Here's a comprehensive list:

- [x] `/health`
  - [x] `GET /`: returns `200 OK` if service is up and running correctly.
- [x] `/queues`
  - [x] `GET /`: gets all queues available returning:
    ```json
    {
      "queues":[
        {
          "id": "blabla",
          "pending_events_count": 120 
        },
        {
          "id": "blabla2",
          "pending_events_count": 110 
        }
      ]
    }
    ```
  - [x] `GET /{queue_id}`: gets basic informations about a specific queue.
  - [x] `POST /`: creates a queue. Must pass a `CreateQueueRequest` json as payload:
        ```json
        {
          "id": "queue_id"
        }
        ```
  - [x] `DELETE /{queue_id}`: deletes a queue by queue name.
- [ ] `/events`
  - [ ] `POST /`: schedules an event. Must pass a `ScheduleEventRequest` json as payload:
    ```json
    {
      "data": "base64 encoded data"
      "schedule_rules": [
        {
          "schedule_at": 1684182908606,
          "recurring_options": {
            "times": 20,
            "delay": 3600000,
          }
        }
      ]
    }
    ```
    - [ ] **`data`**: base64 encoded payload.
    - [ ] **`schedule_rules`** an array of objects contining:
      - [ ] **`schedule_at`**: timestamp at which the event will be sent, must be an unsigned integer. 
      - [ ] **`recurring_options`**:
        - [ ] **`times`**: how many times should the event be scheduled in-between `delays`. Use `-1` to schedule the event infinite times.
        - [ ] **`delay`**: delay in-between event schedules, in milliseconds.
- [ ] `/channels`
  - [ ] `GET /`: retrieves all active channels
  - [ ] `GET /{channel_id}`: returns information about a specific channel
  - [ ] `GET /{channel_id}/events`: polling endpoint.
  - [ ] `POST /`: creates a new channel returning a unique ID and opening it.
  - [ ] `DELETE /{channel_id}`: closes a channel and deletes it.
  - [ ] `PUT /{channel_id}`: edits an active channel settings.

