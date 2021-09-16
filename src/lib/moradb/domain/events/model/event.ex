defmodule Moradb.Event do
  defstruct [
    :id,
    :createdAt,
    :fireAt,
    :category,
    :data
  ]

  @type t :: %__MODULE__{
          id: String.t(),
          createdAt: integer(),
          fireAt: integer(),
          category: String.t(),
          data: Map.t()
        }
end
