FROM python:3.8-slim-bullseye


COPY --from=localhost:5000/dependencies:latest /tmp /tmp


ENV DASH_DEBUG_MODE False

RUN set -ex && pip install \
    confluent-kafka==1.7.0 \
    pandas==1.3.4 \
    dash==2.7.0 \
    gunicorn

COPY charts/7_dashboard/app/ /app/

WORKDIR /app

EXPOSE 8050

# CMD ["gunicorn", "-b", "0.0.0.0:8050", "--reload", "main:server"]

CMD ["python", "main.py"]