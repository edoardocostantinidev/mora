defmodule Moradb.Api do
  use Plug.Router
  plug(:match)
  plug(:dispatch)
  forward("/events", to: Moradb.Routers.Events)
end
