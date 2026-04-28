import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Copy, Edit, Plus, Star, Trash2 } from "lucide-react";

import { Page } from "../components/layout/Page";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card, CardBody } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { useTemplateStore } from "../stores/templateStore";

export function TemplateList() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { templates, loading, error, refresh, remove, duplicate, setDefault } =
    useTemplateStore();

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <Page
      crumbs={["Cabinet Lemaire", t("templates.title")]}
      title={t("templates.title")}
      subtitle={`${templates.length} modèles`}
      actions={
        <Button
          variant="primary"
          leadingIcon={<Plus size={13} strokeWidth={1.5} />}
          onClick={() => navigate("/templates/create")}
        >
          {t("templates.new")}
        </Button>
      }
    >
      {error ? <p className="mb-3 text-[13px] text-danger">{error}</p> : null}
      {loading ? (
        <Card>
          <EmptyState description={t("common.loading")} />
        </Card>
      ) : templates.length === 0 ? (
        <Card>
          <EmptyState description={t("templates.none")} />
        </Card>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {templates.map((tpl) => {
            const accent = tpl.accent_color ?? "var(--color-accent)";
            return (
              <Card key={tpl.id}>
                <div
                  className="h-[200px] bg-paper-2 border-b border-line p-4 flex flex-col gap-1.5"
                  style={{
                    fontFamily:
                      tpl.font_family === "Serif"
                        ? "ui-serif, Georgia, serif"
                        : "var(--font-sans)",
                  }}
                >
                  <div className="flex items-start justify-between">
                    <div
                      className="w-8 h-8"
                      style={{ background: accent, opacity: 0.9 }}
                    />
                    <div className="text-right">
                      <div className="text-[14px] font-semibold tracking-[0.04em]">
                        FACTURE
                      </div>
                      <div className="text-[11px] text-ink-3">#1001</div>
                    </div>
                  </div>
                  <div className="h-1.5 bg-line w-[55%] mt-3" />
                  <div className="h-1 bg-line-soft w-[40%]" />
                  <div className="h-1 bg-line-soft w-[35%]" />
                  <div className="flex-1" />
                  <div className="h-px bg-line" />
                  <div className="flex justify-between">
                    <div className="h-1 bg-line-soft w-[30%]" />
                    <div
                      className="h-1 w-[20%]"
                      style={{ background: accent }}
                    />
                  </div>
                </div>
                <CardBody>
                  <div className="flex items-center justify-between mb-1.5">
                    <div className="font-medium">{tpl.name}</div>
                    {tpl.is_default ? (
                      <Badge kind="final">{t("templates.default")}</Badge>
                    ) : null}
                  </div>
                  <div className="text-[11px] text-ink-3 mb-3">
                    {t(`templates.layout_${tpl.base_layout.toLowerCase()}`)} ·{" "}
                    {t(`templates.font_${tpl.font_family.toLowerCase()}`)}
                    {tpl.accent_color ? (
                      <>
                        {" · "}
                        <span className="font-mono">{tpl.accent_color}</span>
                      </>
                    ) : null}
                  </div>
                  <div className="flex flex-wrap gap-1.5">
                    <Button
                      size="sm"
                      leadingIcon={<Edit size={11} strokeWidth={1.5} />}
                      onClick={() => navigate(`/templates/${tpl.id}/edit`)}
                    >
                      {t("common.edit")}
                    </Button>
                    <Button
                      size="sm"
                      leadingIcon={<Copy size={11} strokeWidth={1.5} />}
                      onClick={() => void duplicate(tpl.id)}
                    >
                      {t("templates.duplicate")}
                    </Button>
                    {!tpl.is_default ? (
                      <Button
                        size="sm"
                        leadingIcon={<Star size={11} strokeWidth={1.5} />}
                        onClick={() => void setDefault(tpl.id)}
                      >
                        {t("templates.make_default")}
                      </Button>
                    ) : null}
                    {!tpl.is_default ? (
                      <Button
                        size="sm"
                        iconOnly
                        variant="danger"
                        aria-label={t("common.delete")}
                        onClick={() => {
                          if (confirm(t("common.confirm_delete"))) {
                            void remove(tpl.id).catch((e) => alert(String(e)));
                          }
                        }}
                      >
                        <Trash2 size={11} strokeWidth={1.5} />
                      </Button>
                    ) : null}
                  </div>
                </CardBody>
              </Card>
            );
          })}
        </div>
      )}
    </Page>
  );
}
