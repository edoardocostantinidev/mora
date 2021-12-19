MORA

mora is a tool capable of streaming scheduled events to every client connected.

## Why mora? 

I always faced myself with tasks that needed to perform some kind of action (sending an email, notify a client, process stuff) but in delayed manner. I would often use `cron` like libraries and tools to achieve this kind of tasks, but I don't like this kind of approach, to me it doesn't seem "scalable".
I recently join [Prima](https://www.prima.it) and as an on-boarding project I decided to give a shot at building a distributed scheduler.


## How mora?

Mora revolves around the concept of `event`.

An `event` is a single piece of information stored in moradb, with some basic properties:
- `id`, unique id (string formatted as follow "{timestamp}-{fireAt}-{eventHash generated by hashing the other fields}")
- `category`, field that describe what category owns the event (string)
- `createdAt` timestamp defining when the event was created (µs)
- `fireAt` timestamp defining when the event should be fired
- `data`, custom data field

Mora is currently developed as an `elixir` application that consists in 4 distinct layers: 

## Alohomora?

![Gif](https://c.tenor.com/sx6rUhrAM1sAAAAC/alohomora-wand.gif)

### API 

The API layer handles all clients requests (e.g. event scheduling). It's basically a single POST endpoint `/events` that accepts an array of `events` such as this one:

```json
"event": {
    "createdAt": 1603140122,
    "fireAt": 1603140222,
    "category": "categoryName",
    "data": {
        "field": "value"
    }
};
```

Once the event is received the api sends it to the **Database** layer and the **Temporal Queue**.

### Database

The database layer, as the name suggest, it's responsible for saving events on disk. Currently database is implemented with `Mnesia` and the `Memento` library. If mora is deployed in a cluster (see **Deploy** section) the database layer will propagate any event it receives to other nodes.

As of right now the database serves as a mere storage tool that gets queried by the **Temporal Queue** layer.

### Temporal Queue

Temporal queue is where the enqueueing and dequeueing logic is performed. For each `category` of events received `mora` spawns a separate `temporal queue`, capable of dispatching events to the **Dispatcher** layer. 
Once an event is notified to a queue if the temporal queue has space it will queue up the event. If the temporal queue is full it will pick it up via query to the database later on, unless the `fireAt` field is inside the current temporal range of the queue. In that case the temporal queue will make room for it by re-processing the last event in the queue at a later (compatible) time. This process will insure that each event is fired correctly and temporal queues are never overqueued. If `mora` is deployed in a cluster, each fired event will trigger a dispatch on other nodes, therefore every client will receive the event.
Each queue can adhere to a behavior therefore differ in dispatching logic, currently the only implementation is based on `Priority`. When the `fireAt` time is `<=` current time, the event gets pushed to a **Dispatcher**.

### Dispatcher

Dispatchers are the last layers of `mora` and they are responsible for sending the events to the connected clients through different implementations. As of right now there is just the **WebSocket** implementation. Each client connects via websocket to `/ws/events/{categoryName}` and listens for events. 

## How to use

the easiest way to use mora is by applying the `kubernetes/mora.yaml` template on a local (or remote) `k8s` cluster. Then use the included `test-client` to test out both event generation and event reception.

in order:

1) `kubectl apply -f kubernetes/mora.yaml`
2) `node test-client/ws.js` and on a separate shell `node test-client/event-generator.js`

As of right now you will receive a lot of debug info on your events, is perfectly normal. Also try and mess around with replicas on your k8s cluster and see what happens. You can event spawn other generators or test via postman. 

If you want to test it out in local make sure to use `dev` environment so that `libcluster` topologies don't interfere with `mora`.

## How to contribute

I'm new to elixir so feel free to create issues and PRs for best practices, naming/code conventions and performance/functionality improvements.

### Test

run `elixirc test/support/event_generator.exs` before `mix test`, tests use an event generator that has to be compiled manually (don't know how to fix it).

### NB

There are a lot of things that should be done differently, being an on-boarding project I couldn't work on it all work day, there are a lot of worst-practices (e.g. secrets on k8s, distributed computation done wrong, no regard whatsoever to backups etc.). With time hopefully I will address each issue and build something good and stable... that probably no one will ever use :D 
