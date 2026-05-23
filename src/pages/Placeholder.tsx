import { useTranslation } from "react-i18next";

interface Props {
  title: string;
}

export function Placeholder({ title }: Props) {
  const { t } = useTranslation();
  return (
    <div className="max-w-2xl">
      <h1 className="mb-2 text-2xl font-bold text-fg">{title}</h1>
      <p className="text-sm text-fg-muted">{t("placeholder.coming_soon")}</p>
    </div>
  );
}
