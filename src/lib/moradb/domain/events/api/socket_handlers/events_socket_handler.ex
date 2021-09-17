defmodule Moradb.Events.SocketHandler do
  @behaviour :cowboy_websocket
  require Logger

  def init(req, _state) do
    [event_category] =
      req.path
      |> String.split(~r/\//)
      |> Enum.take(-1)

    state = %{registry_key: event_category, count: 0}
    IO.inspect(state)
    {:cowboy_websocket, req, state}
  end

  def websocket_init(state) do
    Registry.Moradb
    |> Registry.register(state.registry_key, {})

    {:ok, state}
  end

  def websocket_handle({:text, json}, state) do
    Logger.info("Handling websocket event notification")
    IO.inspect(json)
    IO.inspect(state)
    event = Poison.decode!(json, as: %Moradb.Event{})
    Moradb.Events.Dispatchers.Websocket.dispatch(event)
    new_state = %{registry_key: state.registry_key, count: state.count + 1}
    {:reply, {:text, "#{new_state.count}"}, new_state}
  end

  def websocket_info(info, state) do
    Logger.info("Handling websocket event notification")
    IO.inspect(info)
    IO.inspect(state)
    {:reply, {:text, Poison.encode!(info)}, state}
  end
end
