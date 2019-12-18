type t = {
  constants : ((Id.t * Value.t list) * Value.t option) list;
  domains   : (Id.t * Value.t list) list;
  init      : (Id.t * Value.t) list;
  rules     : (Id.t * Id.t * Rule.t) list;
  variables : (Id.t * Id.t) list;
  views     : (Id.t * Id.t) list;
}
