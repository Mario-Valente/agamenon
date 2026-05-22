{{/*
Expand the name of the chart.
*/}}
{{- define "agamenon.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "agamenon.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "agamenon.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "agamenon.labels" -}}
helm.sh/chart: {{ include "agamenon.chart" . }}
{{ include "agamenon.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "agamenon.selectorLabels" -}}
app.kubernetes.io/name: {{ include "agamenon.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "agamenon.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "agamenon.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Database URL for PostgreSQL backend
*/}}
{{- define "agamenon.databaseUrl" -}}
{{- $host := .Values.storage.postgres.host }}
{{- $port := .Values.storage.postgres.port | default 5432 }}
{{- $database := .Values.storage.postgres.database | default "agamenon" }}
{{- $username := .Values.storage.postgres.username | default "agamenon" }}
{{- $sslMode := .Values.storage.postgres.sslMode | default "require" }}
{{- printf "postgresql://%s:$(POSTGRES_PASSWORD)@%s:%d/%s?sslmode=%s" $username $host (int $port) $database $sslMode }}
{{- end }}
