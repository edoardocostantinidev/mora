# moradb

moradb is an `event` based database capable of streaming future events to every client subscribed to the relative `category`.
An `event` is a single piece of information stored in moradb, with some basic properties:
- `createdAt` timestamp defining when the event was created
- `fireAt` timestamp defining when the event should be fired
- `category`, field that describe what category owns the event
- `id`, unique id
- `data`, custom data field

