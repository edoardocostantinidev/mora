****# API

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
- [x] `/events`
  - [x] `POST /`: schedules an event. Must pass a `ScheduleEventRequest` json as payload:
    ```json
    {
      "data": "base64 encoded data",
      "queue": "test:queue",
      "schedule_rules": [
        {
          "schedule_for": 1684182908606,
          "recurring_options": null | {
            "times": 20,
            "delay": 3600000,
          }
        }
      ]
    }
    ```
    - **`data`**: base64 encoded payload.
    - **`schedule_rules`** an array of objects contining:
      - **`schedule_for`**: timestamp at which the event will be sent, must be an unsigned integer.
      - **`queue_name`**: name of the queue that will host the event.
      - **`recurring_options`**:
        - **`times`**: how many times should the event be scheduled in-between `delays`. Use `-1` to schedule the event infinite times.
        - **`delay`**: delay in-between event schedules, in milliseconds.
- [x] `/channels`
  - [x] `GET /`: retrieves all active channels
  - [x] `GET /{channel_id}`: returns information about a specific channel
  - [x] `GET /{channel_id}/events`: polling endpoint.
  - [x] `POST /`: creates a new channel returning a unique ID and opening it. A payload containing informations about what queues to listen to can be provided, otherwise all events will be listened.
  ```json
  {
    "queues":[
      "queue_1",
      "queue_2"
    ]
  }
  ```
  - [x] `DELETE /{channel_id}`: closes a channel and deletes it.

