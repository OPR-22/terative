import type { ClientAttributeValuesDto } from "../../ipc";

interface Props {
  values: ClientAttributeValuesDto;
}

/// Renders <datalist> elements with IDs `gender-suggestions`,
/// `pronouns-suggestions`, and `occupation-suggestions`. Inputs that want
/// autocomplete just add `list="gender-suggestions"` (etc.). Sex is excluded
/// because it has a fixed dropdown UI; the values are still queried so the
/// API stays uniform, but the form doesn't surface them.
export function ClientAttributeDatalists({ values }: Props) {
  return (
    <>
      <datalist id="gender-suggestions">
        {values.gender.map((v) => (
          <option key={v} value={v} />
        ))}
      </datalist>
      <datalist id="pronouns-suggestions">
        {values.pronouns.map((v) => (
          <option key={v} value={v} />
        ))}
      </datalist>
      <datalist id="occupation-suggestions">
        {values.occupation.map((v) => (
          <option key={v} value={v} />
        ))}
      </datalist>
    </>
  );
}
