import { Index, IndexContext, UnregisteredIndex } from "../core/Index";
import { Update, mapUpdate } from "../core/Update";

export abstract class AggregateIndex<In, Value> extends Index<In, any> {
  abstract value(): Value;
}

export class UnregisteredAggregateIndex<In, Value> extends UnregisteredIndex<AggregateIndex<In, Value>> {
  map<T>(f: (value: Value) => T): UnregisteredAggregateIndex<In, T> {
    return new UnregisteredMapAggregateIndex(ctx => new MapAggregateIndex(ctx, this._register(ctx), (x) => x, f));
  }

  premap<T>(f: (value: T) => In): UnregisteredAggregateIndex<T, Value> {
    return new UnregisteredMapAggregateIndex(ctx => new MapAggregateIndex(ctx, this._register(ctx), f, (x) => x));
  }
}

class MapAggregateIndex<NewIn, NewValue, OldIn, OldValue> extends AggregateIndex<
  NewIn,
  NewValue
> {
  constructor(
    ctx: IndexContext<any>,
    readonly inner: AggregateIndex<OldIn, OldValue>,
    readonly premap: (value: NewIn) => OldIn,
    readonly map: (value: OldValue) => NewValue
  ) {
    super(ctx);
  }

  _onUpdate(update: Update<NewIn>): () => void {
    return this.inner._onUpdate(mapUpdate(this.premap, update));
  }

  override value(): NewValue {
    return this.map(this.inner.value());
  }
}

class UnregisteredMapAggregateIndex<NewIn, NewValue, OldIn, OldValue> extends UnregisteredAggregateIndex<NewIn, NewValue> {
  constructor(
    readonly _register: (ctx: IndexContext<any>) => MapAggregateIndex<NewIn, NewValue, OldIn, OldValue>
  ) {
    super(_register);
  }
  
  override map<NewerValue>(f: (value: NewValue) => NewerValue): UnregisteredMapAggregateIndex<NewIn, NewerValue, OldIn, OldValue> {
    return new UnregisteredMapAggregateIndex(ctx => {
      const ix = this._register(ctx)
      return new MapAggregateIndex(ctx, ix.inner, ix.premap, (x) => f(ix.map(x)));
    })
  }

  override premap<NewerIn>(f: (value: NewerIn) => NewIn): UnregisteredMapAggregateIndex<NewerIn, NewValue, OldIn, OldValue> {
    return new UnregisteredMapAggregateIndex(ctx => {
      const ix = this._register(ctx)
      return new MapAggregateIndex(ctx, ix.inner, (x) => ix.premap(f(x)), ix.map);
    })
  }
}