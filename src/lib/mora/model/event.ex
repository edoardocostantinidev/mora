defmodule Mora.Model.Event do
  @moduledoc """
  Event model.
  """
  use Memento.Table,
    attributes: [
      :id,
      :createdAt,
      :fireAt,
      :category,
      :data
    ],
    index: [:fireAt],
    type: :ordered_set

  @type t :: %__MODULE__{
          id: String.t(),
          createdAt: integer(),
          fireAt: integer(),
          category: String.t(),
          data: Map.t()
        }
end
