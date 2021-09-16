defmodule Moradb.Events.SocketHandler do
  @behaviour :cowboy_websocket
  require Logger

  def init(req, _state) do
    state = %{registry_key: req.path, count: 0}
    {:cowboy_websocket, req, state}
  end

  def websocket_init(state) do
    Registry.Moradb
    |> Registry.register(state.registry_key, {})

    {:ok, state}
  end

  def websocket_handle({:text, json}, state) do
    event = Poison.decode!(json, as: %Moradb.Event{})
    Moradb.Events.Dispatchers.Websocket.dispatch(event)
    new_state = %{registry_key: state.registry_key, count: state.count + 1}
    {:reply, {:text, "#{new_state.count}"}, new_state}
  end

  def websocket_info(info, state) do
    Logger.info("websocket_info")
    IO.inspect(info)
    IO.inspect(state)
    {:reply, {:text, info}, state}
  end
end
