--
-- PostgreSQL database dump
--

\restrict kdQp12cgTM0mCNk7UfgWnsDoR4xjkSiN94fB5T8cKTaDhL3yZdFpaFMnu21ooyT

-- Dumped from database version 15.14 (Debian 15.14-0+deb12u1)
-- Dumped by pg_dump version 15.14 (Debian 15.14-0+deb12u1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: tenant_100002; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA tenant_100002;


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: vector; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public;


--
-- Name: enum_eckwms_instances_tier; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.enum_eckwms_instances_tier AS ENUM (
    'free',
    'paid'
);


--
-- Name: enum_scans_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.enum_scans_status AS ENUM (
    'buffered',
    'delivered',
    'confirmed'
);


--
-- Name: clone_schema(text, text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.clone_schema(source_schema text, dest_schema text) RETURNS void
    LANGUAGE plpgsql
    AS $$
    DECLARE
      object text;
      buffer text;
    BEGIN
      -- Create the new schema
      EXECUTE 'CREATE SCHEMA "' || dest_schema || '"';

      -- Create tables
      FOR object IN
        SELECT table_name::text FROM information_schema.tables WHERE table_schema = source_schema AND table_type = 'BASE TABLE'
      LOOP
        buffer := 'CREATE TABLE "' || dest_schema || '"."' || object || '" (LIKE "' || source_schema || '"."' || object || '" INCLUDING ALL)';
        EXECUTE buffer;
      END LOOP;

      -- Create sequences
      FOR object IN
        SELECT sequence_name::text FROM information_schema.sequences WHERE sequence_schema = source_schema
      LOOP
        buffer := 'CREATE SEQUENCE "' || dest_schema || '"."' || object || '"';
        EXECUTE buffer;
      END LOOP;

    END;
    $$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: active_transaction_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.active_transaction_items (
    id integer NOT NULL,
    active_transaction_id integer NOT NULL,
    item_id integer NOT NULL,
    quantity numeric(10,3) NOT NULL,
    unit_price numeric(10,2) NOT NULL,
    total_price numeric(12,2) NOT NULL,
    tax_rate numeric(5,2) NOT NULL,
    tax_amount numeric(12,2) NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    parent_transaction_item_id integer
);


--
-- Name: active_transaction_items_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.active_transaction_items_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: active_transaction_items_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.active_transaction_items_id_seq OWNED BY public.active_transaction_items.id;


--
-- Name: active_transactions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.active_transactions (
    id integer NOT NULL,
    uuid uuid NOT NULL,
    status character varying(255) DEFAULT 'active'::character varying NOT NULL,
    user_id integer,
    total_amount numeric(12,2) DEFAULT '0'::numeric NOT NULL,
    tax_amount numeric(12,2) DEFAULT '0'::numeric NOT NULL,
    business_date date NOT NULL,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    resolution_status character varying(255) DEFAULT 'none'::character varying,
    payment_type character varying(255),
    payment_amount numeric(12,2),
    bon_start timestamp with time zone,
    bon_end timestamp with time zone,
    bon_nr integer
);


--
-- Name: active_transactions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.active_transactions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: active_transactions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.active_transactions_id_seq OWNED BY public.active_transactions.id;


--
-- Name: ai_agents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_agents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    model_type text NOT NULL,
    model_version text,
    api_key text,
    description text,
    is_active boolean DEFAULT true,
    status text DEFAULT 'active'::text,
    access_tier text DEFAULT 'workflow'::text,
    max_rate_per_min bigint DEFAULT 60,
    max_tokens_per_day bigint DEFAULT 1000000,
    total_requests bigint DEFAULT 0,
    total_tokens_used bigint DEFAULT 0,
    last_request_at timestamp with time zone,
    allowed_ip_ranges text,
    require_approval boolean DEFAULT false,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);


--
-- Name: ai_audit_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_audit_logs (
    id bigint NOT NULL,
    agent_id text NOT NULL,
    function_name text NOT NULL,
    request_data jsonb,
    response_data jsonb,
    status text DEFAULT 'success'::text,
    error_message text,
    execution_time bigint,
    tokens_used bigint,
    ip_address text,
    user_agent text,
    created_at timestamp with time zone
);


--
-- Name: ai_audit_logs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ai_audit_logs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ai_audit_logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ai_audit_logs_id_seq OWNED BY public.ai_audit_logs.id;


--
-- Name: ai_chat_messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_chat_messages (
    id integer NOT NULL,
    "pageId" character varying(255) NOT NULL,
    "userId" uuid,
    "guestName" character varying(100),
    "sessionId" character varying(255),
    message text NOT NULL,
    "isAi" boolean DEFAULT false,
    metadata jsonb DEFAULT '{}'::jsonb,
    "createdAt" timestamp with time zone NOT NULL
);


--
-- Name: ai_chat_messages_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ai_chat_messages_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ai_chat_messages_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ai_chat_messages_id_seq OWNED BY public.ai_chat_messages.id;


--
-- Name: ai_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_permissions (
    id bigint NOT NULL,
    agent_id text NOT NULL,
    function_name text NOT NULL,
    scope text DEFAULT '*'::text,
    max_rate bigint DEFAULT 10,
    is_enabled boolean DEFAULT true,
    allowed_params jsonb,
    denied_params jsonb,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);


--
-- Name: ai_permissions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ai_permissions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ai_permissions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.ai_permissions_id_seq OWNED BY public.ai_permissions.id;


--
-- Name: ai_rate_limits; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_rate_limits (
    agent_id text NOT NULL,
    function_name text NOT NULL,
    window_start timestamp with time zone NOT NULL,
    request_count bigint DEFAULT 0,
    tokens_used bigint DEFAULT 0,
    updated_at timestamp with time zone
);


--
-- Name: branches; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.branches (
    id integer NOT NULL,
    company_id integer,
    branch_name character varying(255) NOT NULL,
    branch_address character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: branches_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.branches_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: branches_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.branches_id_seq OWNED BY public.branches.id;


--
-- Name: categories; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.categories (
    id integer NOT NULL,
    pos_device_id integer,
    source_unique_identifier character varying(255) NOT NULL,
    category_names jsonb NOT NULL,
    category_type character varying(255) NOT NULL,
    parent_category_id integer,
    default_linked_main_group_unique_identifier integer,
    audit_trail jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: categories_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.categories_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: categories_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.categories_id_seq OWNED BY public.categories.id;


--
-- Name: comments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comments (
    id integer NOT NULL,
    "userId" uuid,
    "guestName" character varying(100),
    summary text,
    "isFlagged" boolean DEFAULT false,
    "moderationScore" double precision,
    content text NOT NULL,
    "targetType" character varying(50) NOT NULL,
    "targetId" character varying(255) NOT NULL,
    "parentId" integer,
    "isEdited" boolean DEFAULT false,
    "isDeleted" boolean DEFAULT false,
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL
);


--
-- Name: comments_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.comments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: comments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.comments_id_seq OWNED BY public.comments.id;


--
-- Name: companies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.companies (
    id integer NOT NULL,
    company_full_name character varying(255) NOT NULL,
    meta_information jsonb NOT NULL,
    global_configurations jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: companies_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.companies_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: companies_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.companies_id_seq OWNED BY public.companies.id;


--
-- Name: daily_log_archives; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.daily_log_archives (
    id integer NOT NULL,
    business_date date NOT NULL,
    original_data_hash text NOT NULL,
    data_shards_count integer NOT NULL,
    parity_shards_count integer NOT NULL,
    shards_json text NOT NULL,
    log_ids_json jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: daily_log_archives_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.daily_log_archives_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: daily_log_archives_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.daily_log_archives_id_seq OWNED BY public.daily_log_archives.id;


--
-- Name: delivery_carrier; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.delivery_carrier (
    id bigint NOT NULL,
    name text NOT NULL,
    provider_code text NOT NULL,
    active boolean DEFAULT true,
    config_json text,
    created_at timestamp with time zone,
    updated_at timestamp with time zone
);


--
-- Name: delivery_carrier_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.delivery_carrier_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: delivery_carrier_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.delivery_carrier_id_seq OWNED BY public.delivery_carrier.id;


--
-- Name: delivery_tracking; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.delivery_tracking (
    id bigint NOT NULL,
    picking_delivery_id bigint NOT NULL,
    "timestamp" timestamp with time zone NOT NULL,
    status text NOT NULL,
    status_code text,
    location text,
    description text,
    created_at timestamp with time zone
);


--
-- Name: delivery_tracking_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.delivery_tracking_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: delivery_tracking_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.delivery_tracking_id_seq OWNED BY public.delivery_tracking.id;


--
-- Name: documents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.documents (
    document_id uuid DEFAULT gen_random_uuid() NOT NULL,
    type text NOT NULL,
    status text DEFAULT 'pending'::text,
    payload jsonb,
    device_id text,
    user_id text,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);


--
-- Name: dsfinvk_locations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.dsfinvk_locations (
    location_id integer NOT NULL,
    loc_name character varying(255) NOT NULL,
    loc_strasse character varying(255),
    loc_plz character varying(255),
    loc_ort character varying(255),
    loc_land character varying(255),
    loc_ustid character varying(255),
    pos_device_id integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: dsfinvk_locations_location_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.dsfinvk_locations_location_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: dsfinvk_locations_location_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.dsfinvk_locations_location_id_seq OWNED BY public.dsfinvk_locations.location_id;


--
-- Name: dsfinvk_tse; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.dsfinvk_tse (
    id integer NOT NULL,
    tse_id character varying(255) NOT NULL,
    tse_serial character varying(255) NOT NULL,
    tse_sig_algo character varying(255) NOT NULL,
    tse_zeitformat character varying(255) NOT NULL,
    tse_pd_encoding character varying(255) NOT NULL,
    tse_public_key text NOT NULL,
    tse_zertifikat_i text NOT NULL,
    tse_zertifikat_ii text,
    pos_device_id integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: dsfinvk_tse_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.dsfinvk_tse_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: dsfinvk_tse_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.dsfinvk_tse_id_seq OWNED BY public.dsfinvk_tse.id;


--
-- Name: dsfinvk_vat_mapping; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.dsfinvk_vat_mapping (
    id integer NOT NULL,
    internal_tax_rate numeric(5,2) NOT NULL,
    dsfinvk_ust_schluessel integer NOT NULL,
    description character varying(255) NOT NULL
);


--
-- Name: dsfinvk_vat_mapping_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.dsfinvk_vat_mapping_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: dsfinvk_vat_mapping_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.dsfinvk_vat_mapping_id_seq OWNED BY public.dsfinvk_vat_mapping.id;


--
-- Name: eckwms_instances; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eckwms_instances (
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    server_url character varying(255) NOT NULL,
    api_key character varying(255) NOT NULL,
    tier public.enum_eckwms_instances_tier DEFAULT 'free'::public.enum_eckwms_instances_tier NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL,
    "publicIp" character varying(255),
    "localIps" jsonb,
    "tracerouteToGlobal" text,
    "serverPublicKey" text,
    "lastSeen" timestamp with time zone
);


--
-- Name: encrypted_sync_packets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.encrypted_sync_packets (
    id bigint NOT NULL,
    entity_type text,
    entity_id text,
    version bigint,
    source_instance text,
    vector_clock jsonb,
    key_id text,
    algorithm text,
    encrypted_payload bytea,
    nonce bytea,
    created_at timestamp with time zone,
    updated_at timestamp with time zone
);


--
-- Name: encrypted_sync_packets_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.encrypted_sync_packets_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: encrypted_sync_packets_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.encrypted_sync_packets_id_seq OWNED BY public.encrypted_sync_packets.id;


--
-- Name: entity_checksums; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.entity_checksums (
    id bigint NOT NULL,
    entity_type character varying(50) NOT NULL,
    entity_id character varying(255) NOT NULL,
    content_hash character varying(64) NOT NULL,
    children_hash character varying(64),
    full_hash character varying(64) NOT NULL,
    child_count bigint DEFAULT 0,
    last_updated timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    source_instance character varying(255),
    source_device character varying(255),
    created_at timestamp with time zone,
    updated_at timestamp with time zone
);


--
-- Name: entity_checksums_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.entity_checksums_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: entity_checksums_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.entity_checksums_id_seq OWNED BY public.entity_checksums.id;


--
-- Name: export_jobs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.export_jobs (
    id integer NOT NULL,
    job_id character varying(255) NOT NULL,
    status text DEFAULT 'PENDING'::text NOT NULL,
    export_type text NOT NULL,
    parameters json,
    file_path character varying(255),
    download_token character varying(255),
    error_message text,
    expires_at timestamp with time zone,
    created_by integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT export_jobs_export_type_check CHECK ((export_type = 'dsfinvk'::text)),
    CONSTRAINT export_jobs_status_check CHECK ((status = ANY (ARRAY['PENDING'::text, 'PROCESSING'::text, 'COMPLETE'::text, 'FAILED'::text])))
);


--
-- Name: export_jobs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.export_jobs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: export_jobs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.export_jobs_id_seq OWNED BY public.export_jobs.id;


--
-- Name: file_resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.file_resources (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    file_name text NOT NULL,
    file_path text NOT NULL,
    mime_type character varying(100),
    size bigint,
    checksum character varying(64),
    source_instance character varying(255),
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);


--
-- Name: fiscal_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fiscal_log (
    id integer NOT NULL,
    log_id uuid NOT NULL,
    timestamp_utc timestamp with time zone NOT NULL,
    event_type character varying(255) NOT NULL,
    transaction_number_tse bigint NOT NULL,
    user_id integer,
    payload_for_tse jsonb NOT NULL,
    tse_response jsonb NOT NULL,
    previous_log_hash character varying(255) NOT NULL,
    current_log_hash character varying(255) NOT NULL
);


--
-- Name: fiscal_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.fiscal_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: fiscal_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.fiscal_log_id_seq OWNED BY public.fiscal_log.id;


--
-- Name: item_embeddings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.item_embeddings (
    item_id integer NOT NULL,
    item_embedding public.vector(768),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.items (
    id integer NOT NULL,
    pos_device_id integer,
    source_unique_identifier character varying(255) NOT NULL,
    associated_category_unique_identifier integer,
    display_names jsonb NOT NULL,
    item_price_value numeric(10,2) NOT NULL,
    pricing_schedules jsonb,
    availability_schedule jsonb,
    additional_item_attributes jsonb,
    item_flags jsonb NOT NULL,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    menu_item_number character varying(255)
);


--
-- Name: items_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.items_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: items_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.items_id_seq OWNED BY public.items.id;


--
-- Name: knex_migrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.knex_migrations (
    id integer NOT NULL,
    name character varying(255),
    batch integer,
    migration_time timestamp with time zone
);


--
-- Name: knex_migrations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.knex_migrations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: knex_migrations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.knex_migrations_id_seq OWNED BY public.knex_migrations.id;


--
-- Name: knex_migrations_lock; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.knex_migrations_lock (
    index integer NOT NULL,
    is_locked integer
);


--
-- Name: knex_migrations_lock_index_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.knex_migrations_lock_index_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: knex_migrations_lock_index_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.knex_migrations_lock_index_seq OWNED BY public.knex_migrations_lock.index;


--
-- Name: knowledge_vectors; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.knowledge_vectors (
    id integer NOT NULL,
    content text NOT NULL,
    embedding public.vector(768),
    metadata jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: knowledge_vectors_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.knowledge_vectors_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: knowledge_vectors_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.knowledge_vectors_id_seq OWNED BY public.knowledge_vectors.id;


--
-- Name: menu_layouts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.menu_layouts (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    layout_data jsonb NOT NULL,
    is_active boolean DEFAULT false,
    source_type character varying(255) DEFAULT 'USER_CREATED'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: menu_layouts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.menu_layouts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: menu_layouts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.menu_layouts_id_seq OWNED BY public.menu_layouts.id;


--
-- Name: operational_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.operational_log (
    id integer NOT NULL,
    log_id uuid NOT NULL,
    timestamp_utc timestamp with time zone NOT NULL,
    event_type character varying(255) NOT NULL,
    user_id integer,
    details jsonb,
    previous_log_hash character varying(255) NOT NULL,
    current_log_hash character varying(255) NOT NULL
);


--
-- Name: operational_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.operational_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: operational_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.operational_log_id_seq OWNED BY public.operational_log.id;


--
-- Name: orders; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.orders (
    id bigint NOT NULL,
    order_number text NOT NULL,
    order_type text NOT NULL,
    customer_name text,
    customer_email text,
    customer_phone text,
    item_id bigint,
    product_sku text,
    product_name text,
    serial_number text,
    purchase_date timestamp with time zone,
    issue_description text,
    diagnosis_notes text,
    assigned_to uuid,
    status text DEFAULT 'pending'::text,
    priority text DEFAULT 'normal'::text,
    repair_notes text,
    parts_used jsonb,
    labor_hours numeric,
    total_cost numeric,
    resolution text,
    notes text,
    metadata jsonb,
    rma_reason text,
    is_refund_requested boolean DEFAULT false,
    started_at timestamp with time zone,
    completed_at timestamp with time zone,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);


--
-- Name: orders_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.orders_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: orders_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.orders_id_seq OWNED BY public.orders.id;


--
-- Name: page_contexts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.page_contexts (
    id integer NOT NULL,
    "pageId" character varying(255) NOT NULL,
    "pageName" character varying(255) NOT NULL,
    context text NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb,
    "isActive" boolean DEFAULT true,
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL
);


--
-- Name: page_contexts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.page_contexts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: page_contexts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.page_contexts_id_seq OWNED BY public.page_contexts.id;


--
-- Name: pending_changes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pending_changes (
    id integer NOT NULL,
    change_id character varying(255) NOT NULL,
    requested_by_user_id integer,
    change_type character varying(255) NOT NULL,
    target_entity_type character varying(255) NOT NULL,
    target_entity_id integer,
    original_data jsonb,
    proposed_data jsonb NOT NULL,
    reason text,
    priority character varying(255) DEFAULT 'normal'::character varying,
    status character varying(255) DEFAULT 'pending'::character varying,
    reviewed_by_user_id integer,
    reviewed_at timestamp with time zone,
    review_notes text,
    auto_apply_at timestamp with time zone,
    requires_admin_approval boolean DEFAULT true,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: pending_changes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.pending_changes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: pending_changes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.pending_changes_id_seq OWNED BY public.pending_changes.id;


--
-- Name: pending_fiscal_operations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pending_fiscal_operations (
    id integer NOT NULL,
    operation_id uuid NOT NULL,
    status character varying(255) NOT NULL,
    payload_for_tse jsonb NOT NULL,
    tse_response jsonb,
    last_error text,
    retry_count integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: pending_fiscal_operations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.pending_fiscal_operations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: pending_fiscal_operations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.pending_fiscal_operations_id_seq OWNED BY public.pending_fiscal_operations.id;


--
-- Name: pos_devices; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pos_devices (
    id integer NOT NULL,
    branch_id integer,
    pos_device_name character varying(255) NOT NULL,
    pos_device_type character varying(255) NOT NULL,
    pos_device_external_number integer NOT NULL,
    pos_device_settings jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    kasse_brand character varying(255),
    kasse_modell character varying(255),
    kasse_seriennr character varying(255),
    kasse_sw_brand character varying(255),
    kasse_sw_version character varying(255)
);


--
-- Name: pos_devices_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.pos_devices_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: pos_devices_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.pos_devices_id_seq OWNED BY public.pos_devices.id;


--
-- Name: product_aliases; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.product_aliases (
    id bigint NOT NULL,
    external_code text NOT NULL,
    internal_id text NOT NULL,
    type text NOT NULL,
    is_verified boolean DEFAULT false,
    confidence_score bigint DEFAULT 0,
    created_context text,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);


--
-- Name: product_aliases_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.product_aliases_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: product_aliases_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.product_aliases_id_seq OWNED BY public.product_aliases.id;


--
-- Name: product_product; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.product_product (
    id bigint NOT NULL,
    default_code text,
    barcode text,
    name text,
    active boolean DEFAULT true,
    type text,
    list_price numeric,
    standard_price numeric,
    weight numeric,
    volume numeric,
    write_date timestamp with time zone,
    last_synced_at timestamp with time zone,
    raw_data jsonb
);


--
-- Name: registered_devices; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.registered_devices (
    instance_id uuid,
    is_active boolean DEFAULT true,
    "deviceName" text,
    "createdAt" timestamp with time zone,
    "updatedAt" timestamp with time zone,
    device_id text NOT NULL,
    name text,
    public_key text,
    status text DEFAULT 'pending'::text,
    last_seen_at timestamp with time zone,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone,
    "publicKey" text,
    "lastSeenAt" timestamp with time zone,
    home_instance_id text
);


--
-- Name: res_partner; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.res_partner (
    id bigint NOT NULL,
    name text,
    street text,
    street2 text,
    zip text,
    city text,
    state_id bigint,
    country_id bigint,
    phone text,
    email text,
    vat text,
    company_type text,
    is_company boolean
);


--
-- Name: rma_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rma_requests (
    id uuid NOT NULL,
    "userId" uuid,
    "rmaCode" character varying(255) NOT NULL,
    "orderCode" character varying(255) NOT NULL,
    status character varying(255) DEFAULT 'created'::character varying NOT NULL,
    company character varying(255) NOT NULL,
    person character varying(255),
    street character varying(255) NOT NULL,
    "houseNumber" character varying(255),
    "postalCode" character varying(255) NOT NULL,
    city character varying(255) NOT NULL,
    country character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    "invoiceEmail" character varying(255),
    phone character varying(255),
    "resellerName" character varying(255),
    devices jsonb DEFAULT '[]'::jsonb NOT NULL,
    "orderData" jsonb,
    "receivedAt" timestamp with time zone,
    "processedAt" timestamp with time zone,
    "shippedAt" timestamp with time zone,
    "trackingNumber" character varying(255),
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL
);


--
-- Name: roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.roles (
    id integer NOT NULL,
    role_name character varying(255) NOT NULL,
    role_display_names jsonb NOT NULL,
    description text,
    permissions jsonb NOT NULL,
    default_storno_daily_limit numeric(10,2) DEFAULT '50'::numeric,
    default_storno_emergency_limit numeric(10,2) DEFAULT '25'::numeric,
    can_approve_changes boolean DEFAULT false,
    can_manage_users boolean DEFAULT false,
    is_system_role boolean DEFAULT false,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: roles_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.roles_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: roles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.roles_id_seq OWNED BY public.roles.id;


--
-- Name: scans; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.scans (
    id uuid NOT NULL,
    "deviceId" character varying(255),
    status character varying(255) DEFAULT 'buffered'::character varying NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL,
    payload text DEFAULT '{}'::text,
    checksum character varying(255) DEFAULT ''::character varying,
    instance_id uuid,
    priority integer DEFAULT 0,
    type character varying(255)
);


--
-- Name: search_cache; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.search_cache (
    id integer NOT NULL,
    query_text text NOT NULL,
    query_embedding bytea,
    model_used character varying(255) NOT NULL,
    result_item_ids jsonb NOT NULL,
    full_response_text text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: search_cache_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.search_cache_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: search_cache_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.search_cache_id_seq OWNED BY public.search_cache.id;


--
-- Name: stock_location; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_location (
    id bigint NOT NULL,
    name text,
    complete_name text,
    barcode text,
    usage text,
    location_id bigint,
    active boolean DEFAULT true,
    last_synced_at timestamp with time zone,
    created_at timestamp without time zone,
    updated_at timestamp with time zone
);


--
-- Name: stock_lot; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_lot (
    id bigint NOT NULL,
    name text,
    product_id bigint,
    ref text,
    create_date timestamp with time zone
);


--
-- Name: stock_move_line; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_move_line (
    id bigint NOT NULL,
    picking_id bigint,
    product_id bigint,
    qty_done numeric,
    location_id bigint,
    location_dest_id bigint,
    package_id bigint,
    result_package_id bigint,
    lot_id bigint,
    state text,
    reference text
);


--
-- Name: stock_package_type; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_package_type (
    id bigint NOT NULL,
    name text,
    barcode text,
    max_weight numeric,
    length bigint,
    width bigint,
    height bigint
);


--
-- Name: stock_picking; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_picking (
    id bigint NOT NULL,
    name text,
    state text,
    location_id bigint,
    location_dest_id bigint,
    scheduled_date timestamp with time zone,
    origin text,
    priority text,
    picking_type_id bigint,
    partner_id bigint,
    date_done timestamp with time zone
);


--
-- Name: stock_picking_delivery; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_picking_delivery (
    id bigint NOT NULL,
    picking_id bigint,
    carrier_id bigint,
    tracking_number text,
    carrier_price numeric,
    currency text DEFAULT 'EUR'::text,
    status text DEFAULT 'draft'::text,
    error_message text,
    label_url text,
    label_data bytea,
    raw_response text,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    shipped_at timestamp with time zone,
    delivered_at timestamp with time zone,
    last_activity_at timestamp with time zone
);


--
-- Name: stock_picking_delivery_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.stock_picking_delivery_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: stock_picking_delivery_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.stock_picking_delivery_id_seq OWNED BY public.stock_picking_delivery.id;


--
-- Name: stock_quant; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_quant (
    id bigint NOT NULL,
    product_id bigint,
    location_id bigint,
    lot_id bigint,
    package_id bigint,
    quantity numeric,
    reserved_qty numeric
);


--
-- Name: stock_quant_package; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_quant_package (
    id bigint NOT NULL,
    name text,
    package_type_id bigint,
    location_id bigint,
    pack_date timestamp with time zone
);


--
-- Name: storno_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.storno_log (
    id integer NOT NULL,
    storno_id character varying(255) NOT NULL,
    user_id integer,
    transaction_id character varying(255) NOT NULL,
    storno_amount numeric(10,2) NOT NULL,
    storno_type character varying(255) NOT NULL,
    reason text NOT NULL,
    approved_by_user_id integer,
    within_credit_limit boolean DEFAULT true,
    credit_used numeric(10,2) NOT NULL,
    remaining_credit_after numeric(10,2) NOT NULL,
    approval_status character varying(255) DEFAULT 'automatic'::character varying,
    approved_at timestamp with time zone,
    additional_data jsonb,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: storno_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.storno_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: storno_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.storno_log_id_seq OWNED BY public.storno_log.id;


--
-- Name: sync_conflicts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sync_conflicts (
    id bigint NOT NULL,
    entity_type character varying(100) NOT NULL,
    entity_id character varying(255) NOT NULL,
    conflict_type character varying(50),
    local_data jsonb,
    local_metadata jsonb,
    remote_data jsonb,
    remote_metadata jsonb,
    auto_resolution_strategy character varying(50),
    auto_resolution_winner character varying(50),
    manual_resolution jsonb,
    status character varying(50) DEFAULT 'pending'::character varying,
    resolved_at timestamp with time zone,
    resolved_by character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: sync_conflicts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.sync_conflicts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: sync_conflicts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.sync_conflicts_id_seq OWNED BY public.sync_conflicts.id;


--
-- Name: sync_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sync_history (
    provider text NOT NULL,
    status text NOT NULL,
    started_at timestamp with time zone NOT NULL,
    completed_at timestamp with time zone,
    duration bigint DEFAULT 0,
    created bigint DEFAULT 0,
    updated bigint DEFAULT 0,
    skipped bigint DEFAULT 0,
    errors bigint DEFAULT 0,
    error_detail text,
    debug_info jsonb,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone,
    instance_id character varying(255),
    id character varying(36) NOT NULL
);


--
-- Name: sync_metadata; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sync_metadata (
    id bigint NOT NULL,
    instance_id character varying(255) NOT NULL,
    entity_type character varying(100) NOT NULL,
    last_sync_at timestamp with time zone,
    last_full_sync_at timestamp with time zone,
    last_sync_status character varying(50),
    records_synced bigint DEFAULT 0,
    records_conflicts bigint DEFAULT 0,
    sync_duration_ms bigint,
    vector_clock jsonb DEFAULT '{}'::jsonb,
    error_message text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: sync_metadata_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.sync_metadata_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: sync_metadata_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.sync_metadata_id_seq OWNED BY public.sync_metadata.id;


--
-- Name: sync_queue; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sync_queue (
    id bigint NOT NULL,
    entity_type character varying(100) NOT NULL,
    entity_id character varying(255) NOT NULL,
    operation character varying(20) NOT NULL,
    payload jsonb,
    metadata jsonb,
    priority bigint DEFAULT 5,
    retry_count bigint DEFAULT 0,
    max_retries bigint DEFAULT 3,
    scheduled_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    processed_at timestamp with time zone,
    status character varying(50) DEFAULT 'pending'::character varying,
    error_message text,
    target_instance character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: sync_queue_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.sync_queue_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: sync_queue_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.sync_queue_id_seq OWNED BY public.sync_queue.id;


--
-- Name: sync_routes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sync_routes (
    id bigint NOT NULL,
    instance_id character varying(255) NOT NULL,
    route_url character varying(500) NOT NULL,
    route_type character varying(50) NOT NULL,
    is_active boolean DEFAULT true,
    last_success_at timestamp with time zone,
    last_failure_at timestamp with time zone,
    success_count bigint DEFAULT 0,
    failure_count bigint DEFAULT 0,
    avg_latency_ms bigint,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: sync_routes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.sync_routes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: sync_routes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.sync_routes_id_seq OWNED BY public.sync_routes.id;


--
-- Name: system_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.system_log (
    id integer NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    level character varying(255) NOT NULL,
    message text NOT NULL,
    context jsonb
);


--
-- Name: system_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.system_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: system_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.system_log_id_seq OWNED BY public.system_log.id;


--
-- Name: tenant_access_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tenant_access_keys (
    id integer NOT NULL,
    company_id integer NOT NULL,
    key_hash character varying(255) NOT NULL,
    description character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    last_used_at timestamp with time zone,
    is_active boolean DEFAULT true
);


--
-- Name: tenant_access_keys_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tenant_access_keys_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tenant_access_keys_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tenant_access_keys_id_seq OWNED BY public.tenant_access_keys.id;


--
-- Name: translation_cache; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.translation_cache (
    key character varying(32) NOT NULL,
    language character varying(5) NOT NULL,
    "originalText" text NOT NULL,
    "translatedText" text NOT NULL,
    context character varying(100),
    "lastUsed" timestamp with time zone,
    "useCount" integer DEFAULT 1,
    "charCount" integer DEFAULT 0,
    "processingTime" double precision,
    source character varying(20) DEFAULT 'openai'::character varying,
    "apiVersion" character varying(30),
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL
);


--
-- Name: user_auth; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_auth (
    id uuid NOT NULL,
    username character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    password character varying(255),
    "googleId" character varying(255),
    name character varying(255),
    company character varying(255),
    phone character varying(255),
    street character varying(255),
    "houseNumber" character varying(255),
    "postalCode" character varying(255),
    city character varying(255),
    country character varying(255),
    "lastLogin" timestamp with time zone,
    role character varying(255) DEFAULT 'user'::character varying,
    "userType" character varying(255) DEFAULT 'individual'::character varying,
    "rmaReference" character varying(255),
    "isActive" boolean DEFAULT true,
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL
);


--
-- Name: user_auths; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_auths (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    username text NOT NULL,
    password text NOT NULL,
    email text NOT NULL,
    name text,
    role text DEFAULT 'user'::text,
    user_type text DEFAULT 'individual'::text,
    company text,
    google_id text,
    is_active boolean DEFAULT true,
    last_login timestamp with time zone,
    failed_login_attempts bigint DEFAULT 0,
    preferred_language text DEFAULT 'en'::text,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone,
    pin text DEFAULT ''::text
);


--
-- Name: user_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_sessions (
    id integer NOT NULL,
    session_id character varying(255) NOT NULL,
    user_id integer,
    expires_at timestamp with time zone NOT NULL,
    ip_address character varying(255),
    user_agent character varying(255),
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: user_sessions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_sessions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_sessions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_sessions_id_seq OWNED BY public.user_sessions.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id integer NOT NULL,
    username character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    password_hash character varying(255) NOT NULL,
    full_name character varying(255) NOT NULL,
    role_id integer,
    pos_device_id integer,
    storno_daily_limit numeric(10,2) NOT NULL,
    storno_emergency_limit numeric(10,2) NOT NULL,
    storno_used_today numeric(10,2) DEFAULT '0'::numeric,
    trust_score integer DEFAULT 50,
    is_active boolean DEFAULT true,
    force_password_change boolean DEFAULT false,
    last_login_at timestamp with time zone,
    last_login_ip character varying(255),
    failed_login_attempts integer DEFAULT 0,
    locked_until timestamp with time zone,
    user_preferences jsonb,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    bediener_id character varying(255),
    CONSTRAINT users_trust_score_check CHECK (((trust_score >= 0) AND (trust_score <= 100)))
);


--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: vec_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.vec_items (
    id integer NOT NULL,
    item_embedding real[],
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: vec_items_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.vec_items_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: vec_items_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.vec_items_id_seq OWNED BY public.vec_items.id;


--
-- Name: warehouse_racks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.warehouse_racks (
    id bigint NOT NULL,
    name character varying(100) NOT NULL,
    prefix character varying(10),
    columns bigint DEFAULT 1 NOT NULL,
    rows bigint DEFAULT 1 NOT NULL,
    start_index bigint NOT NULL,
    sort_order bigint DEFAULT 0,
    warehouse_id bigint,
    pos_x bigint DEFAULT 0,
    pos_y bigint DEFAULT 0,
    rotation bigint DEFAULT 0,
    visual_width bigint DEFAULT 0,
    visual_height bigint DEFAULT 0,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    mapped_location_id bigint,
    deleted_at timestamp with time zone
);


--
-- Name: warehouse_racks_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.warehouse_racks_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: warehouse_racks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.warehouse_racks_id_seq OWNED BY public.warehouse_racks.id;


--
-- Name: active_transaction_items; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.active_transaction_items (
    id integer DEFAULT nextval('public.active_transaction_items_id_seq'::regclass) NOT NULL,
    active_transaction_id integer NOT NULL,
    item_id integer NOT NULL,
    quantity numeric(10,3) NOT NULL,
    unit_price numeric(10,2) NOT NULL,
    total_price numeric(12,2) NOT NULL,
    tax_rate numeric(5,2) NOT NULL,
    tax_amount numeric(12,2) NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    parent_transaction_item_id integer
);


--
-- Name: active_transaction_items_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.active_transaction_items_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: active_transactions; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.active_transactions (
    id integer DEFAULT nextval('public.active_transactions_id_seq'::regclass) NOT NULL,
    uuid uuid NOT NULL,
    status character varying(255) DEFAULT 'active'::character varying NOT NULL,
    user_id integer,
    total_amount numeric(12,2) DEFAULT '0'::numeric NOT NULL,
    tax_amount numeric(12,2) DEFAULT '0'::numeric NOT NULL,
    business_date date NOT NULL,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    resolution_status character varying(255) DEFAULT 'none'::character varying,
    payment_type character varying(255),
    payment_amount numeric(12,2),
    bon_start timestamp with time zone,
    bon_end timestamp with time zone,
    bon_nr integer
);


--
-- Name: active_transactions_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.active_transactions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: branches; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.branches (
    id integer DEFAULT nextval('public.branches_id_seq'::regclass) NOT NULL,
    company_id integer,
    branch_name character varying(255) NOT NULL,
    branch_address character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: branches_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.branches_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: categories; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.categories (
    id integer DEFAULT nextval('public.categories_id_seq'::regclass) NOT NULL,
    pos_device_id integer,
    source_unique_identifier character varying(255) NOT NULL,
    category_names jsonb NOT NULL,
    category_type character varying(255) NOT NULL,
    parent_category_id integer,
    default_linked_main_group_unique_identifier integer,
    audit_trail jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: categories_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.categories_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: companies; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.companies (
    id integer DEFAULT nextval('public.companies_id_seq'::regclass) NOT NULL,
    company_full_name character varying(255) NOT NULL,
    meta_information jsonb NOT NULL,
    global_configurations jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: companies_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.companies_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: daily_log_archives; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.daily_log_archives (
    id integer DEFAULT nextval('public.daily_log_archives_id_seq'::regclass) NOT NULL,
    business_date date NOT NULL,
    original_data_hash text NOT NULL,
    data_shards_count integer NOT NULL,
    parity_shards_count integer NOT NULL,
    shards_json text NOT NULL,
    log_ids_json jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: daily_log_archives_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.daily_log_archives_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: dsfinvk_locations; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.dsfinvk_locations (
    location_id integer DEFAULT nextval('public.dsfinvk_locations_location_id_seq'::regclass) NOT NULL,
    loc_name character varying(255) NOT NULL,
    loc_strasse character varying(255),
    loc_plz character varying(255),
    loc_ort character varying(255),
    loc_land character varying(255),
    loc_ustid character varying(255),
    pos_device_id integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: dsfinvk_locations_location_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.dsfinvk_locations_location_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: dsfinvk_tse; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.dsfinvk_tse (
    id integer DEFAULT nextval('public.dsfinvk_tse_id_seq'::regclass) NOT NULL,
    tse_id character varying(255) NOT NULL,
    tse_serial character varying(255) NOT NULL,
    tse_sig_algo character varying(255) NOT NULL,
    tse_zeitformat character varying(255) NOT NULL,
    tse_pd_encoding character varying(255) NOT NULL,
    tse_public_key text NOT NULL,
    tse_zertifikat_i text NOT NULL,
    tse_zertifikat_ii text,
    pos_device_id integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: dsfinvk_tse_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.dsfinvk_tse_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: dsfinvk_vat_mapping; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.dsfinvk_vat_mapping (
    id integer DEFAULT nextval('public.dsfinvk_vat_mapping_id_seq'::regclass) NOT NULL,
    internal_tax_rate numeric(5,2) NOT NULL,
    dsfinvk_ust_schluessel integer NOT NULL,
    description character varying(255) NOT NULL
);


--
-- Name: dsfinvk_vat_mapping_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.dsfinvk_vat_mapping_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: export_jobs; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.export_jobs (
    id integer DEFAULT nextval('public.export_jobs_id_seq'::regclass) NOT NULL,
    job_id character varying(255) NOT NULL,
    status text DEFAULT 'PENDING'::text NOT NULL,
    export_type text NOT NULL,
    parameters json,
    file_path character varying(255),
    download_token character varying(255),
    error_message text,
    expires_at timestamp with time zone,
    created_by integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT export_jobs_export_type_check CHECK ((export_type = 'dsfinvk'::text)),
    CONSTRAINT export_jobs_status_check CHECK ((status = ANY (ARRAY['PENDING'::text, 'PROCESSING'::text, 'COMPLETE'::text, 'FAILED'::text])))
);


--
-- Name: export_jobs_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.export_jobs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: fiscal_log; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.fiscal_log (
    id integer DEFAULT nextval('public.fiscal_log_id_seq'::regclass) NOT NULL,
    log_id uuid NOT NULL,
    timestamp_utc timestamp with time zone NOT NULL,
    event_type character varying(255) NOT NULL,
    transaction_number_tse bigint NOT NULL,
    user_id integer,
    payload_for_tse jsonb NOT NULL,
    tse_response jsonb NOT NULL,
    previous_log_hash character varying(255) NOT NULL,
    current_log_hash character varying(255) NOT NULL
);


--
-- Name: fiscal_log_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.fiscal_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: item_embeddings; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.item_embeddings (
    item_id integer NOT NULL,
    item_embedding public.vector(768),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: items; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.items (
    id integer DEFAULT nextval('public.items_id_seq'::regclass) NOT NULL,
    pos_device_id integer,
    source_unique_identifier character varying(255) NOT NULL,
    associated_category_unique_identifier integer,
    display_names jsonb NOT NULL,
    item_price_value numeric(10,2) NOT NULL,
    pricing_schedules jsonb,
    availability_schedule jsonb,
    additional_item_attributes jsonb,
    item_flags jsonb NOT NULL,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    menu_item_number character varying(255)
);


--
-- Name: items_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.items_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: knex_migrations; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.knex_migrations (
    id integer DEFAULT nextval('public.knex_migrations_id_seq'::regclass) NOT NULL,
    name character varying(255),
    batch integer,
    migration_time timestamp with time zone
);


--
-- Name: knex_migrations_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.knex_migrations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: knex_migrations_lock; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.knex_migrations_lock (
    index integer DEFAULT nextval('public.knex_migrations_lock_index_seq'::regclass) NOT NULL,
    is_locked integer
);


--
-- Name: knex_migrations_lock_index_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.knex_migrations_lock_index_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: knowledge_vectors; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.knowledge_vectors (
    id integer DEFAULT nextval('public.knowledge_vectors_id_seq'::regclass) NOT NULL,
    content text NOT NULL,
    embedding public.vector(768),
    metadata jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: knowledge_vectors_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.knowledge_vectors_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: menu_layouts; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.menu_layouts (
    id integer DEFAULT nextval('public.menu_layouts_id_seq'::regclass) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    layout_data jsonb NOT NULL,
    is_active boolean DEFAULT false,
    source_type character varying(255) DEFAULT 'USER_CREATED'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: menu_layouts_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.menu_layouts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: operational_log; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.operational_log (
    id integer DEFAULT nextval('public.operational_log_id_seq'::regclass) NOT NULL,
    log_id uuid NOT NULL,
    timestamp_utc timestamp with time zone NOT NULL,
    event_type character varying(255) NOT NULL,
    user_id integer,
    details jsonb,
    previous_log_hash character varying(255) NOT NULL,
    current_log_hash character varying(255) NOT NULL
);


--
-- Name: operational_log_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.operational_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: pending_changes; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.pending_changes (
    id integer DEFAULT nextval('public.pending_changes_id_seq'::regclass) NOT NULL,
    change_id character varying(255) NOT NULL,
    requested_by_user_id integer,
    change_type character varying(255) NOT NULL,
    target_entity_type character varying(255) NOT NULL,
    target_entity_id integer,
    original_data jsonb,
    proposed_data jsonb NOT NULL,
    reason text,
    priority character varying(255) DEFAULT 'normal'::character varying,
    status character varying(255) DEFAULT 'pending'::character varying,
    reviewed_by_user_id integer,
    reviewed_at timestamp with time zone,
    review_notes text,
    auto_apply_at timestamp with time zone,
    requires_admin_approval boolean DEFAULT true,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: pending_changes_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.pending_changes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: pending_fiscal_operations; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.pending_fiscal_operations (
    id integer DEFAULT nextval('public.pending_fiscal_operations_id_seq'::regclass) NOT NULL,
    operation_id uuid NOT NULL,
    status character varying(255) NOT NULL,
    payload_for_tse jsonb NOT NULL,
    tse_response jsonb,
    last_error text,
    retry_count integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: pending_fiscal_operations_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.pending_fiscal_operations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: pos_devices; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.pos_devices (
    id integer DEFAULT nextval('public.pos_devices_id_seq'::regclass) NOT NULL,
    branch_id integer,
    pos_device_name character varying(255) NOT NULL,
    pos_device_type character varying(255) NOT NULL,
    pos_device_external_number integer NOT NULL,
    pos_device_settings jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    kasse_brand character varying(255),
    kasse_modell character varying(255),
    kasse_seriennr character varying(255),
    kasse_sw_brand character varying(255),
    kasse_sw_version character varying(255)
);


--
-- Name: pos_devices_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.pos_devices_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rma_requests; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.rma_requests (
    id uuid NOT NULL,
    "userId" uuid,
    "rmaCode" character varying(255) NOT NULL,
    "orderCode" character varying(255) NOT NULL,
    status character varying(255) DEFAULT 'created'::character varying NOT NULL,
    company character varying(255) NOT NULL,
    person character varying(255),
    street character varying(255) NOT NULL,
    "houseNumber" character varying(255),
    "postalCode" character varying(255) NOT NULL,
    city character varying(255) NOT NULL,
    country character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    "invoiceEmail" character varying(255),
    phone character varying(255),
    "resellerName" character varying(255),
    devices jsonb DEFAULT '[]'::jsonb NOT NULL,
    "orderData" jsonb,
    "receivedAt" timestamp with time zone,
    "processedAt" timestamp with time zone,
    "shippedAt" timestamp with time zone,
    "trackingNumber" character varying(255),
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL
);


--
-- Name: roles; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.roles (
    id integer DEFAULT nextval('public.roles_id_seq'::regclass) NOT NULL,
    role_name character varying(255) NOT NULL,
    role_display_names jsonb NOT NULL,
    description text,
    permissions jsonb NOT NULL,
    default_storno_daily_limit numeric(10,2) DEFAULT '50'::numeric,
    default_storno_emergency_limit numeric(10,2) DEFAULT '25'::numeric,
    can_approve_changes boolean DEFAULT false,
    can_manage_users boolean DEFAULT false,
    is_system_role boolean DEFAULT false,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: roles_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.roles_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: search_cache; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.search_cache (
    id integer DEFAULT nextval('public.search_cache_id_seq'::regclass) NOT NULL,
    query_text text NOT NULL,
    query_embedding bytea,
    model_used character varying(255) NOT NULL,
    result_item_ids jsonb NOT NULL,
    full_response_text text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: search_cache_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.search_cache_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: storno_log; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.storno_log (
    id integer DEFAULT nextval('public.storno_log_id_seq'::regclass) NOT NULL,
    storno_id character varying(255) NOT NULL,
    user_id integer,
    transaction_id character varying(255) NOT NULL,
    storno_amount numeric(10,2) NOT NULL,
    storno_type character varying(255) NOT NULL,
    reason text NOT NULL,
    approved_by_user_id integer,
    within_credit_limit boolean DEFAULT true,
    credit_used numeric(10,2) NOT NULL,
    remaining_credit_after numeric(10,2) NOT NULL,
    approval_status character varying(255) DEFAULT 'automatic'::character varying,
    approved_at timestamp with time zone,
    additional_data jsonb,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: storno_log_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.storno_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: system_log; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.system_log (
    id integer DEFAULT nextval('public.system_log_id_seq'::regclass) NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    level character varying(255) NOT NULL,
    message text NOT NULL,
    context jsonb
);


--
-- Name: system_log_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.system_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tenant_access_keys; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.tenant_access_keys (
    id integer DEFAULT nextval('public.tenant_access_keys_id_seq'::regclass) NOT NULL,
    company_id integer NOT NULL,
    key_hash character varying(255) NOT NULL,
    description character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    last_used_at timestamp with time zone,
    is_active boolean DEFAULT true
);


--
-- Name: tenant_access_keys_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.tenant_access_keys_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: translation_cache; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.translation_cache (
    key character varying(32) NOT NULL,
    language character varying(5) NOT NULL,
    "originalText" text NOT NULL,
    "translatedText" text NOT NULL,
    context character varying(100),
    "lastUsed" timestamp with time zone,
    "useCount" integer DEFAULT 1,
    "charCount" integer DEFAULT 0,
    "processingTime" double precision,
    source character varying(20) DEFAULT 'openai'::character varying,
    "apiVersion" character varying(30),
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL
);


--
-- Name: user_auth; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.user_auth (
    id uuid NOT NULL,
    username character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    password character varying(255),
    "googleId" character varying(255),
    name character varying(255),
    company character varying(255),
    phone character varying(255),
    street character varying(255),
    "houseNumber" character varying(255),
    "postalCode" character varying(255),
    city character varying(255),
    country character varying(255),
    "lastLogin" timestamp with time zone,
    role character varying(255) DEFAULT 'user'::character varying,
    "userType" character varying(255) DEFAULT 'individual'::character varying,
    "rmaReference" character varying(255),
    "isActive" boolean DEFAULT true,
    "createdAt" timestamp with time zone NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL
);


--
-- Name: user_sessions; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.user_sessions (
    id integer DEFAULT nextval('public.user_sessions_id_seq'::regclass) NOT NULL,
    session_id character varying(255) NOT NULL,
    user_id integer,
    expires_at timestamp with time zone NOT NULL,
    ip_address character varying(255),
    user_agent character varying(255),
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: user_sessions_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.user_sessions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.users (
    id integer DEFAULT nextval('public.users_id_seq'::regclass) NOT NULL,
    username character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    password_hash character varying(255) NOT NULL,
    full_name character varying(255) NOT NULL,
    role_id integer,
    pos_device_id integer,
    storno_daily_limit numeric(10,2) NOT NULL,
    storno_emergency_limit numeric(10,2) NOT NULL,
    storno_used_today numeric(10,2) DEFAULT '0'::numeric,
    trust_score integer DEFAULT 50,
    is_active boolean DEFAULT true,
    force_password_change boolean DEFAULT false,
    last_login_at timestamp with time zone,
    last_login_ip character varying(255),
    failed_login_attempts integer DEFAULT 0,
    locked_until timestamp with time zone,
    user_preferences jsonb,
    audit_trail jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    bediener_id character varying(255),
    CONSTRAINT users_trust_score_check CHECK (((trust_score >= 0) AND (trust_score <= 100)))
);


--
-- Name: users_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: vec_items; Type: TABLE; Schema: tenant_100002; Owner: -
--

CREATE TABLE tenant_100002.vec_items (
    id integer DEFAULT nextval('public.vec_items_id_seq'::regclass) NOT NULL,
    item_embedding real[],
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: vec_items_id_seq; Type: SEQUENCE; Schema: tenant_100002; Owner: -
--

CREATE SEQUENCE tenant_100002.vec_items_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: active_transaction_items id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transaction_items ALTER COLUMN id SET DEFAULT nextval('public.active_transaction_items_id_seq'::regclass);


--
-- Name: active_transactions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transactions ALTER COLUMN id SET DEFAULT nextval('public.active_transactions_id_seq'::regclass);


--
-- Name: ai_audit_logs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_audit_logs ALTER COLUMN id SET DEFAULT nextval('public.ai_audit_logs_id_seq'::regclass);


--
-- Name: ai_chat_messages id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_chat_messages ALTER COLUMN id SET DEFAULT nextval('public.ai_chat_messages_id_seq'::regclass);


--
-- Name: ai_permissions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_permissions ALTER COLUMN id SET DEFAULT nextval('public.ai_permissions_id_seq'::regclass);


--
-- Name: branches id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.branches ALTER COLUMN id SET DEFAULT nextval('public.branches_id_seq'::regclass);


--
-- Name: categories id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.categories ALTER COLUMN id SET DEFAULT nextval('public.categories_id_seq'::regclass);


--
-- Name: comments id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments ALTER COLUMN id SET DEFAULT nextval('public.comments_id_seq'::regclass);


--
-- Name: companies id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.companies ALTER COLUMN id SET DEFAULT nextval('public.companies_id_seq'::regclass);


--
-- Name: daily_log_archives id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.daily_log_archives ALTER COLUMN id SET DEFAULT nextval('public.daily_log_archives_id_seq'::regclass);


--
-- Name: delivery_carrier id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.delivery_carrier ALTER COLUMN id SET DEFAULT nextval('public.delivery_carrier_id_seq'::regclass);


--
-- Name: delivery_tracking id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.delivery_tracking ALTER COLUMN id SET DEFAULT nextval('public.delivery_tracking_id_seq'::regclass);


--
-- Name: dsfinvk_locations location_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_locations ALTER COLUMN location_id SET DEFAULT nextval('public.dsfinvk_locations_location_id_seq'::regclass);


--
-- Name: dsfinvk_tse id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_tse ALTER COLUMN id SET DEFAULT nextval('public.dsfinvk_tse_id_seq'::regclass);


--
-- Name: dsfinvk_vat_mapping id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_vat_mapping ALTER COLUMN id SET DEFAULT nextval('public.dsfinvk_vat_mapping_id_seq'::regclass);


--
-- Name: encrypted_sync_packets id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.encrypted_sync_packets ALTER COLUMN id SET DEFAULT nextval('public.encrypted_sync_packets_id_seq'::regclass);


--
-- Name: entity_checksums id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_checksums ALTER COLUMN id SET DEFAULT nextval('public.entity_checksums_id_seq'::regclass);


--
-- Name: export_jobs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_jobs ALTER COLUMN id SET DEFAULT nextval('public.export_jobs_id_seq'::regclass);


--
-- Name: fiscal_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fiscal_log ALTER COLUMN id SET DEFAULT nextval('public.fiscal_log_id_seq'::regclass);


--
-- Name: items id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.items ALTER COLUMN id SET DEFAULT nextval('public.items_id_seq'::regclass);


--
-- Name: knex_migrations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.knex_migrations ALTER COLUMN id SET DEFAULT nextval('public.knex_migrations_id_seq'::regclass);


--
-- Name: knex_migrations_lock index; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.knex_migrations_lock ALTER COLUMN index SET DEFAULT nextval('public.knex_migrations_lock_index_seq'::regclass);


--
-- Name: knowledge_vectors id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.knowledge_vectors ALTER COLUMN id SET DEFAULT nextval('public.knowledge_vectors_id_seq'::regclass);


--
-- Name: menu_layouts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.menu_layouts ALTER COLUMN id SET DEFAULT nextval('public.menu_layouts_id_seq'::regclass);


--
-- Name: operational_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operational_log ALTER COLUMN id SET DEFAULT nextval('public.operational_log_id_seq'::regclass);


--
-- Name: orders id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders ALTER COLUMN id SET DEFAULT nextval('public.orders_id_seq'::regclass);


--
-- Name: page_contexts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.page_contexts ALTER COLUMN id SET DEFAULT nextval('public.page_contexts_id_seq'::regclass);


--
-- Name: pending_changes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_changes ALTER COLUMN id SET DEFAULT nextval('public.pending_changes_id_seq'::regclass);


--
-- Name: pending_fiscal_operations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_fiscal_operations ALTER COLUMN id SET DEFAULT nextval('public.pending_fiscal_operations_id_seq'::regclass);


--
-- Name: pos_devices id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pos_devices ALTER COLUMN id SET DEFAULT nextval('public.pos_devices_id_seq'::regclass);


--
-- Name: product_aliases id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_aliases ALTER COLUMN id SET DEFAULT nextval('public.product_aliases_id_seq'::regclass);


--
-- Name: roles id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.roles ALTER COLUMN id SET DEFAULT nextval('public.roles_id_seq'::regclass);


--
-- Name: search_cache id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.search_cache ALTER COLUMN id SET DEFAULT nextval('public.search_cache_id_seq'::regclass);


--
-- Name: stock_picking_delivery id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_picking_delivery ALTER COLUMN id SET DEFAULT nextval('public.stock_picking_delivery_id_seq'::regclass);


--
-- Name: storno_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storno_log ALTER COLUMN id SET DEFAULT nextval('public.storno_log_id_seq'::regclass);


--
-- Name: sync_conflicts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_conflicts ALTER COLUMN id SET DEFAULT nextval('public.sync_conflicts_id_seq'::regclass);


--
-- Name: sync_metadata id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_metadata ALTER COLUMN id SET DEFAULT nextval('public.sync_metadata_id_seq'::regclass);


--
-- Name: sync_queue id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_queue ALTER COLUMN id SET DEFAULT nextval('public.sync_queue_id_seq'::regclass);


--
-- Name: sync_routes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_routes ALTER COLUMN id SET DEFAULT nextval('public.sync_routes_id_seq'::regclass);


--
-- Name: system_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_log ALTER COLUMN id SET DEFAULT nextval('public.system_log_id_seq'::regclass);


--
-- Name: tenant_access_keys id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tenant_access_keys ALTER COLUMN id SET DEFAULT nextval('public.tenant_access_keys_id_seq'::regclass);


--
-- Name: user_sessions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions ALTER COLUMN id SET DEFAULT nextval('public.user_sessions_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: vec_items id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vec_items ALTER COLUMN id SET DEFAULT nextval('public.vec_items_id_seq'::regclass);


--
-- Name: warehouse_racks id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.warehouse_racks ALTER COLUMN id SET DEFAULT nextval('public.warehouse_racks_id_seq'::regclass);


--
-- Name: active_transaction_items active_transaction_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transaction_items
    ADD CONSTRAINT active_transaction_items_pkey PRIMARY KEY (id);


--
-- Name: active_transactions active_transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transactions
    ADD CONSTRAINT active_transactions_pkey PRIMARY KEY (id);


--
-- Name: active_transactions active_transactions_uuid_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transactions
    ADD CONSTRAINT active_transactions_uuid_unique UNIQUE (uuid);


--
-- Name: ai_agents ai_agents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_agents
    ADD CONSTRAINT ai_agents_pkey PRIMARY KEY (id);


--
-- Name: ai_audit_logs ai_audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_audit_logs
    ADD CONSTRAINT ai_audit_logs_pkey PRIMARY KEY (id);


--
-- Name: ai_chat_messages ai_chat_messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_chat_messages
    ADD CONSTRAINT ai_chat_messages_pkey PRIMARY KEY (id);


--
-- Name: ai_permissions ai_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_permissions
    ADD CONSTRAINT ai_permissions_pkey PRIMARY KEY (id);


--
-- Name: ai_rate_limits ai_rate_limits_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_rate_limits
    ADD CONSTRAINT ai_rate_limits_pkey PRIMARY KEY (agent_id, function_name, window_start);


--
-- Name: branches branches_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.branches
    ADD CONSTRAINT branches_pkey PRIMARY KEY (id);


--
-- Name: categories categories_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT categories_pkey PRIMARY KEY (id);


--
-- Name: categories categories_pos_device_source_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT categories_pos_device_source_unique UNIQUE (pos_device_id, source_unique_identifier);


--
-- Name: comments comments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_pkey PRIMARY KEY (id);


--
-- Name: companies companies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.companies
    ADD CONSTRAINT companies_pkey PRIMARY KEY (id);


--
-- Name: daily_log_archives daily_log_archives_business_date_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.daily_log_archives
    ADD CONSTRAINT daily_log_archives_business_date_unique UNIQUE (business_date);


--
-- Name: daily_log_archives daily_log_archives_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.daily_log_archives
    ADD CONSTRAINT daily_log_archives_pkey PRIMARY KEY (id);


--
-- Name: delivery_carrier delivery_carrier_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.delivery_carrier
    ADD CONSTRAINT delivery_carrier_pkey PRIMARY KEY (id);


--
-- Name: delivery_tracking delivery_tracking_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.delivery_tracking
    ADD CONSTRAINT delivery_tracking_pkey PRIMARY KEY (id);


--
-- Name: documents documents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_pkey PRIMARY KEY (document_id);


--
-- Name: dsfinvk_locations dsfinvk_locations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_locations
    ADD CONSTRAINT dsfinvk_locations_pkey PRIMARY KEY (location_id);


--
-- Name: dsfinvk_tse dsfinvk_tse_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_tse
    ADD CONSTRAINT dsfinvk_tse_pkey PRIMARY KEY (id);


--
-- Name: dsfinvk_tse dsfinvk_tse_tse_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_tse
    ADD CONSTRAINT dsfinvk_tse_tse_id_unique UNIQUE (tse_id);


--
-- Name: dsfinvk_vat_mapping dsfinvk_vat_mapping_internal_tax_rate_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_vat_mapping
    ADD CONSTRAINT dsfinvk_vat_mapping_internal_tax_rate_unique UNIQUE (internal_tax_rate);


--
-- Name: dsfinvk_vat_mapping dsfinvk_vat_mapping_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_vat_mapping
    ADD CONSTRAINT dsfinvk_vat_mapping_pkey PRIMARY KEY (id);


--
-- Name: eckwms_instances eckwms_instances_api_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key1 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key10; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key10 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key11; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key11 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key12; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key12 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key13; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key13 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key14; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key14 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key15; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key15 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key16; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key16 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key17; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key17 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key18; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key18 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key19; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key19 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key2; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key2 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key20; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key20 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key21; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key21 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key22; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key22 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key23; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key23 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key24; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key24 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key25; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key25 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key26; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key26 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key27; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key27 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key28; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key28 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key29; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key29 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key3; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key3 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key30; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key30 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key31; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key31 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key32; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key32 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key33; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key33 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key34; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key34 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key35; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key35 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key36; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key36 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key37; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key37 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key38; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key38 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key39; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key39 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key4; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key4 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key40; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key40 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key41; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key41 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key5; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key5 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key6; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key6 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key7; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key7 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key8; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key8 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_api_key_key9; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_api_key_key9 UNIQUE (api_key);


--
-- Name: eckwms_instances eckwms_instances_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key1 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key10; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key10 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key11; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key11 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key12; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key12 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key13; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key13 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key14; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key14 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key15; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key15 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key16; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key16 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key17; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key17 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key18; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key18 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key19; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key19 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key2; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key2 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key20; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key20 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key21; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key21 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key22; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key22 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key23; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key23 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key24; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key24 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key25; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key25 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key26; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key26 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key27; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key27 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key28; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key28 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key29; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key29 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key3; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key3 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key30; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key30 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key31; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key31 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key32; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key32 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key33; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key33 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key34; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key34 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key35; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key35 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key36; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key36 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key37; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key37 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key38; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key38 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key39; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key39 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key4; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key4 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key40; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key40 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key41; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key41 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key5; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key5 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key6; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key6 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key7; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key7 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key8; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key8 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_name_key9; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_name_key9 UNIQUE (name);


--
-- Name: eckwms_instances eckwms_instances_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eckwms_instances
    ADD CONSTRAINT eckwms_instances_pkey PRIMARY KEY (id);


--
-- Name: encrypted_sync_packets encrypted_sync_packets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.encrypted_sync_packets
    ADD CONSTRAINT encrypted_sync_packets_pkey PRIMARY KEY (id);


--
-- Name: entity_checksums entity_checksums_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.entity_checksums
    ADD CONSTRAINT entity_checksums_pkey PRIMARY KEY (id);


--
-- Name: export_jobs export_jobs_download_token_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_jobs
    ADD CONSTRAINT export_jobs_download_token_unique UNIQUE (download_token);


--
-- Name: export_jobs export_jobs_job_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_jobs
    ADD CONSTRAINT export_jobs_job_id_unique UNIQUE (job_id);


--
-- Name: export_jobs export_jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_jobs
    ADD CONSTRAINT export_jobs_pkey PRIMARY KEY (id);


--
-- Name: file_resources file_resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_resources
    ADD CONSTRAINT file_resources_pkey PRIMARY KEY (id);


--
-- Name: fiscal_log fiscal_log_current_log_hash_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fiscal_log
    ADD CONSTRAINT fiscal_log_current_log_hash_unique UNIQUE (current_log_hash);


--
-- Name: fiscal_log fiscal_log_log_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fiscal_log
    ADD CONSTRAINT fiscal_log_log_id_unique UNIQUE (log_id);


--
-- Name: fiscal_log fiscal_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fiscal_log
    ADD CONSTRAINT fiscal_log_pkey PRIMARY KEY (id);


--
-- Name: item_embeddings item_embeddings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.item_embeddings
    ADD CONSTRAINT item_embeddings_pkey PRIMARY KEY (item_id);


--
-- Name: items items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.items
    ADD CONSTRAINT items_pkey PRIMARY KEY (id);


--
-- Name: items items_pos_device_source_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.items
    ADD CONSTRAINT items_pos_device_source_unique UNIQUE (pos_device_id, source_unique_identifier);


--
-- Name: knex_migrations_lock knex_migrations_lock_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.knex_migrations_lock
    ADD CONSTRAINT knex_migrations_lock_pkey PRIMARY KEY (index);


--
-- Name: knex_migrations knex_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.knex_migrations
    ADD CONSTRAINT knex_migrations_pkey PRIMARY KEY (id);


--
-- Name: knowledge_vectors knowledge_vectors_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.knowledge_vectors
    ADD CONSTRAINT knowledge_vectors_pkey PRIMARY KEY (id);


--
-- Name: menu_layouts menu_layouts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.menu_layouts
    ADD CONSTRAINT menu_layouts_pkey PRIMARY KEY (id);


--
-- Name: operational_log operational_log_current_log_hash_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operational_log
    ADD CONSTRAINT operational_log_current_log_hash_unique UNIQUE (current_log_hash);


--
-- Name: operational_log operational_log_log_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operational_log
    ADD CONSTRAINT operational_log_log_id_unique UNIQUE (log_id);


--
-- Name: operational_log operational_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operational_log
    ADD CONSTRAINT operational_log_pkey PRIMARY KEY (id);


--
-- Name: orders orders_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);


--
-- Name: page_contexts page_contexts_pageId_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.page_contexts
    ADD CONSTRAINT "page_contexts_pageId_key" UNIQUE ("pageId");


--
-- Name: page_contexts page_contexts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.page_contexts
    ADD CONSTRAINT page_contexts_pkey PRIMARY KEY (id);


--
-- Name: pending_changes pending_changes_change_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_changes
    ADD CONSTRAINT pending_changes_change_id_unique UNIQUE (change_id);


--
-- Name: pending_changes pending_changes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_changes
    ADD CONSTRAINT pending_changes_pkey PRIMARY KEY (id);


--
-- Name: pending_fiscal_operations pending_fiscal_operations_operation_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_fiscal_operations
    ADD CONSTRAINT pending_fiscal_operations_operation_id_unique UNIQUE (operation_id);


--
-- Name: pending_fiscal_operations pending_fiscal_operations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_fiscal_operations
    ADD CONSTRAINT pending_fiscal_operations_pkey PRIMARY KEY (id);


--
-- Name: pos_devices pos_devices_kasse_seriennr_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pos_devices
    ADD CONSTRAINT pos_devices_kasse_seriennr_unique UNIQUE (kasse_seriennr);


--
-- Name: pos_devices pos_devices_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pos_devices
    ADD CONSTRAINT pos_devices_pkey PRIMARY KEY (id);


--
-- Name: product_aliases product_aliases_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_aliases
    ADD CONSTRAINT product_aliases_pkey PRIMARY KEY (id);


--
-- Name: product_product product_product_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_product
    ADD CONSTRAINT product_product_pkey PRIMARY KEY (id);


--
-- Name: registered_devices registered_devices_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.registered_devices
    ADD CONSTRAINT registered_devices_pkey PRIMARY KEY (device_id);


--
-- Name: res_partner res_partner_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.res_partner
    ADD CONSTRAINT res_partner_pkey PRIMARY KEY (id);


--
-- Name: rma_requests rma_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT rma_requests_pkey PRIMARY KEY (id);


--
-- Name: rma_requests rma_requests_rmaCode_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key1" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key10; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key10" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key11; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key11" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key12; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key12" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key13; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key13" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key14; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key14" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key15; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key15" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key16; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key16" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key17; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key17" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key18; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key18" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key19; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key19" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key2; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key2" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key20; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key20" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key21; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key21" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key22; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key22" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key23; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key23" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key24; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key24" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key25; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key25" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key26; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key26" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key27; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key27" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key28; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key28" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key29; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key29" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key3; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key3" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key30; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key30" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key31; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key31" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key32; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key32" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key33; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key33" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key34; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key34" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key35; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key35" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key36; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key36" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key37; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key37" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key38; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key38" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key39; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key39" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key4; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key4" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key40; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key40" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key41; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key41" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key42; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key42" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key43; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key43" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key44; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key44" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key45; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key45" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key46; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key46" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key47; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key47" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key48; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key48" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key49; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key49" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key5; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key5" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key50; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key50" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key51; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key51" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key52; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key52" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key53; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key53" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key54; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key54" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key55; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key55" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key56; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key56" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key57; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key57" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key58; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key58" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key59; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key59" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key6; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key6" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key60; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key60" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key61; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key61" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key62; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key62" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key63; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key63" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key7; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key7" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key8; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key8" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key9; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key9" UNIQUE ("rmaCode");


--
-- Name: roles roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT roles_pkey PRIMARY KEY (id);


--
-- Name: roles roles_role_name_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT roles_role_name_unique UNIQUE (role_name);


--
-- Name: scans scans_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.scans
    ADD CONSTRAINT scans_pkey PRIMARY KEY (id);


--
-- Name: search_cache search_cache_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.search_cache
    ADD CONSTRAINT search_cache_pkey PRIMARY KEY (id);


--
-- Name: stock_location stock_location_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_location
    ADD CONSTRAINT stock_location_pkey PRIMARY KEY (id);


--
-- Name: stock_lot stock_lot_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_lot
    ADD CONSTRAINT stock_lot_pkey PRIMARY KEY (id);


--
-- Name: stock_move_line stock_move_line_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_move_line
    ADD CONSTRAINT stock_move_line_pkey PRIMARY KEY (id);


--
-- Name: stock_package_type stock_package_type_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_package_type
    ADD CONSTRAINT stock_package_type_pkey PRIMARY KEY (id);


--
-- Name: stock_picking_delivery stock_picking_delivery_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_picking_delivery
    ADD CONSTRAINT stock_picking_delivery_pkey PRIMARY KEY (id);


--
-- Name: stock_picking stock_picking_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_picking
    ADD CONSTRAINT stock_picking_pkey PRIMARY KEY (id);


--
-- Name: stock_quant_package stock_quant_package_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_quant_package
    ADD CONSTRAINT stock_quant_package_pkey PRIMARY KEY (id);


--
-- Name: stock_quant stock_quant_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_quant
    ADD CONSTRAINT stock_quant_pkey PRIMARY KEY (id);


--
-- Name: storno_log storno_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storno_log
    ADD CONSTRAINT storno_log_pkey PRIMARY KEY (id);


--
-- Name: storno_log storno_log_storno_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storno_log
    ADD CONSTRAINT storno_log_storno_id_unique UNIQUE (storno_id);


--
-- Name: sync_conflicts sync_conflicts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_conflicts
    ADD CONSTRAINT sync_conflicts_pkey PRIMARY KEY (id);


--
-- Name: sync_history sync_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_history
    ADD CONSTRAINT sync_history_pkey PRIMARY KEY (id);


--
-- Name: sync_metadata sync_metadata_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_metadata
    ADD CONSTRAINT sync_metadata_pkey PRIMARY KEY (id);


--
-- Name: sync_queue sync_queue_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_queue
    ADD CONSTRAINT sync_queue_pkey PRIMARY KEY (id);


--
-- Name: sync_routes sync_routes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_routes
    ADD CONSTRAINT sync_routes_pkey PRIMARY KEY (id);


--
-- Name: system_log system_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_log
    ADD CONSTRAINT system_log_pkey PRIMARY KEY (id);


--
-- Name: tenant_access_keys tenant_access_keys_key_hash_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tenant_access_keys
    ADD CONSTRAINT tenant_access_keys_key_hash_unique UNIQUE (key_hash);


--
-- Name: tenant_access_keys tenant_access_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tenant_access_keys
    ADD CONSTRAINT tenant_access_keys_pkey PRIMARY KEY (id);


--
-- Name: translation_cache translation_cache_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.translation_cache
    ADD CONSTRAINT translation_cache_pkey PRIMARY KEY (key, language);


--
-- Name: user_auths uni_user_auths_email; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auths
    ADD CONSTRAINT uni_user_auths_email UNIQUE (email);


--
-- Name: user_auths uni_user_auths_username; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auths
    ADD CONSTRAINT uni_user_auths_username UNIQUE (username);


--
-- Name: user_auth user_auth_email_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key UNIQUE (email);


--
-- Name: user_auth user_auth_email_key1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key1 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key10; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key10 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key11; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key11 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key12; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key12 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key13; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key13 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key14; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key14 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key15; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key15 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key16; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key16 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key17; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key17 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key18; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key18 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key19; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key19 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key2; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key2 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key20; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key20 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key21; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key21 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key22; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key22 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key23; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key23 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key24; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key24 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key25; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key25 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key26; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key26 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key27; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key27 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key28; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key28 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key29; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key29 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key3; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key3 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key30; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key30 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key31; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key31 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key32; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key32 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key33; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key33 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key34; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key34 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key35; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key35 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key36; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key36 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key37; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key37 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key38; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key38 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key39; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key39 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key4; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key4 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key40; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key40 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key41; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key41 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key42; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key42 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key43; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key43 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key44; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key44 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key45; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key45 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key46; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key46 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key47; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key47 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key48; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key48 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key49; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key49 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key5; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key5 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key50; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key50 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key51; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key51 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key52; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key52 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key53; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key53 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key54; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key54 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key55; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key55 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key56; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key56 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key57; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key57 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key58; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key58 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key59; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key59 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key6; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key6 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key60; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key60 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key61; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key61 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key62; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key62 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key63; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key63 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key64; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key64 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key7; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key7 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key8; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key8 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key9; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_email_key9 UNIQUE (email);


--
-- Name: user_auth user_auth_googleId_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key1" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key10; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key10" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key11; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key11" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key12; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key12" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key13; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key13" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key14; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key14" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key15; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key15" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key16; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key16" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key17; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key17" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key18; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key18" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key19; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key19" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key2; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key2" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key20; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key20" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key21; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key21" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key22; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key22" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key23; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key23" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key24; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key24" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key25; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key25" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key26; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key26" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key27; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key27" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key28; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key28" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key29; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key29" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key3; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key3" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key30; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key30" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key31; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key31" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key32; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key32" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key33; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key33" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key34; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key34" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key35; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key35" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key36; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key36" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key37; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key37" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key38; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key38" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key39; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key39" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key4; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key4" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key40; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key40" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key41; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key41" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key42; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key42" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key43; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key43" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key44; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key44" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key45; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key45" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key46; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key46" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key47; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key47" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key48; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key48" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key49; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key49" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key5; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key5" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key50; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key50" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key51; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key51" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key52; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key52" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key53; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key53" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key54; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key54" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key55; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key55" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key56; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key56" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key57; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key57" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key58; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key58" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key59; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key59" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key6; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key6" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key60; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key60" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key61; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key61" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key62; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key62" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key63; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key63" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key64; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key64" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key7; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key7" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key8; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key8" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key9; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_googleId_key9" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_pkey PRIMARY KEY (id);


--
-- Name: user_auth user_auth_rmaReference_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key1" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key10; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key10" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key11; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key11" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key12; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key12" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key13; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key13" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key14; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key14" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key15; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key15" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key16; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key16" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key17; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key17" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key18; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key18" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key19; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key19" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key2; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key2" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key20; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key20" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key21; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key21" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key22; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key22" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key23; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key23" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key24; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key24" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key25; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key25" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key26; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key26" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key27; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key27" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key28; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key28" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key29; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key29" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key3; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key3" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key30; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key30" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key31; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key31" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key32; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key32" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key33; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key33" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key34; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key34" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key35; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key35" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key36; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key36" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key37; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key37" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key38; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key38" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key39; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key39" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key4; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key4" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key40; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key40" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key41; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key41" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key42; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key42" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key43; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key43" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key44; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key44" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key45; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key45" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key46; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key46" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key47; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key47" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key48; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key48" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key49; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key49" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key5; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key5" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key50; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key50" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key51; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key51" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key52; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key52" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key53; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key53" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key54; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key54" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key55; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key55" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key56; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key56" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key57; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key57" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key58; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key58" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key59; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key59" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key6; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key6" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key60; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key60" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key61; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key61" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key62; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key62" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key63; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key63" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key7; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key7" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key8; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key8" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key9; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key9" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_username_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key UNIQUE (username);


--
-- Name: user_auth user_auth_username_key1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key1 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key10; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key10 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key11; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key11 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key12; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key12 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key13; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key13 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key14; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key14 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key15; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key15 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key16; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key16 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key17; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key17 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key18; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key18 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key19; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key19 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key2; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key2 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key20; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key20 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key21; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key21 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key22; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key22 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key23; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key23 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key24; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key24 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key25; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key25 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key26; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key26 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key27; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key27 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key28; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key28 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key29; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key29 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key3; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key3 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key30; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key30 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key31; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key31 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key32; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key32 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key33; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key33 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key34; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key34 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key35; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key35 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key36; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key36 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key37; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key37 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key38; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key38 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key39; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key39 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key4; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key4 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key40; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key40 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key41; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key41 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key42; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key42 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key43; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key43 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key44; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key44 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key45; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key45 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key46; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key46 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key47; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key47 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key48; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key48 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key49; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key49 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key5; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key5 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key50; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key50 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key51; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key51 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key52; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key52 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key53; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key53 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key54; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key54 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key55; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key55 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key56; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key56 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key57; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key57 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key58; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key58 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key59; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key59 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key6; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key6 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key60; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key60 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key61; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key61 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key62; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key62 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key63; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key63 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key64; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key64 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key7; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key7 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key8; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key8 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key9; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auth
    ADD CONSTRAINT user_auth_username_key9 UNIQUE (username);


--
-- Name: user_auths user_auths_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_auths
    ADD CONSTRAINT user_auths_pkey PRIMARY KEY (id);


--
-- Name: user_sessions user_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_pkey PRIMARY KEY (id);


--
-- Name: user_sessions user_sessions_session_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_session_id_unique UNIQUE (session_id);


--
-- Name: users users_bediener_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_bediener_id_unique UNIQUE (bediener_id);


--
-- Name: users users_email_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_unique UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_username_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_unique UNIQUE (username);


--
-- Name: vec_items vec_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vec_items
    ADD CONSTRAINT vec_items_pkey PRIMARY KEY (id);


--
-- Name: warehouse_racks warehouse_racks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.warehouse_racks
    ADD CONSTRAINT warehouse_racks_pkey PRIMARY KEY (id);


--
-- Name: active_transaction_items active_transaction_items_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.active_transaction_items
    ADD CONSTRAINT active_transaction_items_pkey PRIMARY KEY (id);


--
-- Name: active_transactions active_transactions_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.active_transactions
    ADD CONSTRAINT active_transactions_pkey PRIMARY KEY (id);


--
-- Name: active_transactions active_transactions_uuid_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.active_transactions
    ADD CONSTRAINT active_transactions_uuid_key UNIQUE (uuid);


--
-- Name: branches branches_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.branches
    ADD CONSTRAINT branches_pkey PRIMARY KEY (id);


--
-- Name: categories categories_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.categories
    ADD CONSTRAINT categories_pkey PRIMARY KEY (id);


--
-- Name: categories categories_pos_device_id_source_unique_identifier_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.categories
    ADD CONSTRAINT categories_pos_device_id_source_unique_identifier_key UNIQUE (pos_device_id, source_unique_identifier);


--
-- Name: companies companies_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.companies
    ADD CONSTRAINT companies_pkey PRIMARY KEY (id);


--
-- Name: daily_log_archives daily_log_archives_business_date_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.daily_log_archives
    ADD CONSTRAINT daily_log_archives_business_date_key UNIQUE (business_date);


--
-- Name: daily_log_archives daily_log_archives_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.daily_log_archives
    ADD CONSTRAINT daily_log_archives_pkey PRIMARY KEY (id);


--
-- Name: dsfinvk_locations dsfinvk_locations_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.dsfinvk_locations
    ADD CONSTRAINT dsfinvk_locations_pkey PRIMARY KEY (location_id);


--
-- Name: dsfinvk_tse dsfinvk_tse_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.dsfinvk_tse
    ADD CONSTRAINT dsfinvk_tse_pkey PRIMARY KEY (id);


--
-- Name: dsfinvk_tse dsfinvk_tse_tse_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.dsfinvk_tse
    ADD CONSTRAINT dsfinvk_tse_tse_id_key UNIQUE (tse_id);


--
-- Name: dsfinvk_vat_mapping dsfinvk_vat_mapping_internal_tax_rate_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.dsfinvk_vat_mapping
    ADD CONSTRAINT dsfinvk_vat_mapping_internal_tax_rate_key UNIQUE (internal_tax_rate);


--
-- Name: dsfinvk_vat_mapping dsfinvk_vat_mapping_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.dsfinvk_vat_mapping
    ADD CONSTRAINT dsfinvk_vat_mapping_pkey PRIMARY KEY (id);


--
-- Name: export_jobs export_jobs_download_token_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.export_jobs
    ADD CONSTRAINT export_jobs_download_token_key UNIQUE (download_token);


--
-- Name: export_jobs export_jobs_job_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.export_jobs
    ADD CONSTRAINT export_jobs_job_id_key UNIQUE (job_id);


--
-- Name: export_jobs export_jobs_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.export_jobs
    ADD CONSTRAINT export_jobs_pkey PRIMARY KEY (id);


--
-- Name: fiscal_log fiscal_log_current_log_hash_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.fiscal_log
    ADD CONSTRAINT fiscal_log_current_log_hash_key UNIQUE (current_log_hash);


--
-- Name: fiscal_log fiscal_log_log_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.fiscal_log
    ADD CONSTRAINT fiscal_log_log_id_key UNIQUE (log_id);


--
-- Name: fiscal_log fiscal_log_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.fiscal_log
    ADD CONSTRAINT fiscal_log_pkey PRIMARY KEY (id);


--
-- Name: item_embeddings item_embeddings_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.item_embeddings
    ADD CONSTRAINT item_embeddings_pkey PRIMARY KEY (item_id);


--
-- Name: items items_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.items
    ADD CONSTRAINT items_pkey PRIMARY KEY (id);


--
-- Name: items items_pos_device_id_source_unique_identifier_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.items
    ADD CONSTRAINT items_pos_device_id_source_unique_identifier_key UNIQUE (pos_device_id, source_unique_identifier);


--
-- Name: knex_migrations_lock knex_migrations_lock_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.knex_migrations_lock
    ADD CONSTRAINT knex_migrations_lock_pkey PRIMARY KEY (index);


--
-- Name: knex_migrations knex_migrations_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.knex_migrations
    ADD CONSTRAINT knex_migrations_pkey PRIMARY KEY (id);


--
-- Name: knowledge_vectors knowledge_vectors_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.knowledge_vectors
    ADD CONSTRAINT knowledge_vectors_pkey PRIMARY KEY (id);


--
-- Name: menu_layouts menu_layouts_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.menu_layouts
    ADD CONSTRAINT menu_layouts_pkey PRIMARY KEY (id);


--
-- Name: operational_log operational_log_current_log_hash_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.operational_log
    ADD CONSTRAINT operational_log_current_log_hash_key UNIQUE (current_log_hash);


--
-- Name: operational_log operational_log_log_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.operational_log
    ADD CONSTRAINT operational_log_log_id_key UNIQUE (log_id);


--
-- Name: operational_log operational_log_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.operational_log
    ADD CONSTRAINT operational_log_pkey PRIMARY KEY (id);


--
-- Name: pending_changes pending_changes_change_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.pending_changes
    ADD CONSTRAINT pending_changes_change_id_key UNIQUE (change_id);


--
-- Name: pending_changes pending_changes_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.pending_changes
    ADD CONSTRAINT pending_changes_pkey PRIMARY KEY (id);


--
-- Name: pending_fiscal_operations pending_fiscal_operations_operation_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.pending_fiscal_operations
    ADD CONSTRAINT pending_fiscal_operations_operation_id_key UNIQUE (operation_id);


--
-- Name: pending_fiscal_operations pending_fiscal_operations_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.pending_fiscal_operations
    ADD CONSTRAINT pending_fiscal_operations_pkey PRIMARY KEY (id);


--
-- Name: pos_devices pos_devices_kasse_seriennr_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.pos_devices
    ADD CONSTRAINT pos_devices_kasse_seriennr_key UNIQUE (kasse_seriennr);


--
-- Name: pos_devices pos_devices_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.pos_devices
    ADD CONSTRAINT pos_devices_pkey PRIMARY KEY (id);


--
-- Name: rma_requests rma_requests_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT rma_requests_pkey PRIMARY KEY (id);


--
-- Name: rma_requests rma_requests_rmaCode_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key1; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key1" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key2; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key2" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key3; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key3" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key4; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key4" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key5; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key5" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key6; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key6" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key7; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key7" UNIQUE ("rmaCode");


--
-- Name: rma_requests rma_requests_rmaCode_key8; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.rma_requests
    ADD CONSTRAINT "rma_requests_rmaCode_key8" UNIQUE ("rmaCode");


--
-- Name: roles roles_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.roles
    ADD CONSTRAINT roles_pkey PRIMARY KEY (id);


--
-- Name: roles roles_role_name_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.roles
    ADD CONSTRAINT roles_role_name_key UNIQUE (role_name);


--
-- Name: search_cache search_cache_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.search_cache
    ADD CONSTRAINT search_cache_pkey PRIMARY KEY (id);


--
-- Name: storno_log storno_log_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.storno_log
    ADD CONSTRAINT storno_log_pkey PRIMARY KEY (id);


--
-- Name: storno_log storno_log_storno_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.storno_log
    ADD CONSTRAINT storno_log_storno_id_key UNIQUE (storno_id);


--
-- Name: system_log system_log_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.system_log
    ADD CONSTRAINT system_log_pkey PRIMARY KEY (id);


--
-- Name: tenant_access_keys tenant_access_keys_key_hash_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.tenant_access_keys
    ADD CONSTRAINT tenant_access_keys_key_hash_key UNIQUE (key_hash);


--
-- Name: tenant_access_keys tenant_access_keys_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.tenant_access_keys
    ADD CONSTRAINT tenant_access_keys_pkey PRIMARY KEY (id);


--
-- Name: translation_cache translation_cache_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.translation_cache
    ADD CONSTRAINT translation_cache_pkey PRIMARY KEY (key, language);


--
-- Name: user_auth user_auth_email_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key UNIQUE (email);


--
-- Name: user_auth user_auth_email_key1; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key1 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key2; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key2 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key3; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key3 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key4; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key4 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key5; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key5 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key6; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key6 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key7; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key7 UNIQUE (email);


--
-- Name: user_auth user_auth_email_key8; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_email_key8 UNIQUE (email);


--
-- Name: user_auth user_auth_googleId_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key1; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key1" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key2; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key2" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key3; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key3" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key4; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key4" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key5; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key5" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key6; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key6" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key7; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key7" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_googleId_key8; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_googleId_key8" UNIQUE ("googleId");


--
-- Name: user_auth user_auth_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_pkey PRIMARY KEY (id);


--
-- Name: user_auth user_auth_rmaReference_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key1; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key1" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key2; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key2" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key3; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key3" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key4; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key4" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key5; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key5" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key6; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key6" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key7; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key7" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_rmaReference_key8; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT "user_auth_rmaReference_key8" UNIQUE ("rmaReference");


--
-- Name: user_auth user_auth_username_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key UNIQUE (username);


--
-- Name: user_auth user_auth_username_key1; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key1 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key2; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key2 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key3; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key3 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key4; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key4 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key5; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key5 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key6; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key6 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key7; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key7 UNIQUE (username);


--
-- Name: user_auth user_auth_username_key8; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_auth
    ADD CONSTRAINT user_auth_username_key8 UNIQUE (username);


--
-- Name: user_sessions user_sessions_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_sessions
    ADD CONSTRAINT user_sessions_pkey PRIMARY KEY (id);


--
-- Name: user_sessions user_sessions_session_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.user_sessions
    ADD CONSTRAINT user_sessions_session_id_key UNIQUE (session_id);


--
-- Name: users users_bediener_id_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.users
    ADD CONSTRAINT users_bediener_id_key UNIQUE (bediener_id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: vec_items vec_items_pkey; Type: CONSTRAINT; Schema: tenant_100002; Owner: -
--

ALTER TABLE ONLY tenant_100002.vec_items
    ADD CONSTRAINT vec_items_pkey PRIMARY KEY (id);


--
-- Name: active_transaction_items_active_transaction_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX active_transaction_items_active_transaction_id_index ON public.active_transaction_items USING btree (active_transaction_id);


--
-- Name: active_transactions_business_date_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX active_transactions_business_date_index ON public.active_transactions USING btree (business_date);


--
-- Name: active_transactions_resolution_status_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX active_transactions_resolution_status_index ON public.active_transactions USING btree (resolution_status);


--
-- Name: active_transactions_status_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX active_transactions_status_index ON public.active_transactions USING btree (status);


--
-- Name: active_transactions_uuid_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX active_transactions_uuid_index ON public.active_transactions USING btree (uuid);


--
-- Name: categories_source_unique_identifier_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX categories_source_unique_identifier_index ON public.categories USING btree (source_unique_identifier);


--
-- Name: daily_log_archives_business_date_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX daily_log_archives_business_date_index ON public.daily_log_archives USING btree (business_date);


--
-- Name: dsfinvk_vat_mapping_dsfinvk_ust_schluessel_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX dsfinvk_vat_mapping_dsfinvk_ust_schluessel_index ON public.dsfinvk_vat_mapping USING btree (dsfinvk_ust_schluessel);


--
-- Name: export_jobs_job_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX export_jobs_job_id_index ON public.export_jobs USING btree (job_id);


--
-- Name: fiscal_log_current_log_hash_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX fiscal_log_current_log_hash_index ON public.fiscal_log USING btree (current_log_hash);


--
-- Name: fiscal_log_event_type_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX fiscal_log_event_type_index ON public.fiscal_log USING btree (event_type);


--
-- Name: fiscal_log_log_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX fiscal_log_log_id_index ON public.fiscal_log USING btree (log_id);


--
-- Name: fiscal_log_previous_log_hash_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX fiscal_log_previous_log_hash_index ON public.fiscal_log USING btree (previous_log_hash);


--
-- Name: fiscal_log_timestamp_utc_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX fiscal_log_timestamp_utc_index ON public.fiscal_log USING btree (timestamp_utc);


--
-- Name: fiscal_log_transaction_number_tse_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX fiscal_log_transaction_number_tse_index ON public.fiscal_log USING btree (transaction_number_tse);


--
-- Name: idx_ai_agents_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_agents_deleted_at ON public.ai_agents USING btree (deleted_at);


--
-- Name: idx_ai_audit_logs_agent_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_audit_logs_agent_id ON public.ai_audit_logs USING btree (agent_id);


--
-- Name: idx_ai_audit_logs_function_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_audit_logs_function_name ON public.ai_audit_logs USING btree (function_name);


--
-- Name: idx_ai_chat_page_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_chat_page_time ON public.ai_chat_messages USING btree ("pageId", "createdAt");


--
-- Name: idx_ai_chat_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_chat_session ON public.ai_chat_messages USING btree ("sessionId");


--
-- Name: idx_ai_chat_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_chat_user ON public.ai_chat_messages USING btree ("userId");


--
-- Name: idx_ai_permissions_agent_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_permissions_agent_id ON public.ai_permissions USING btree (agent_id);


--
-- Name: idx_ai_permissions_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_permissions_deleted_at ON public.ai_permissions USING btree (deleted_at);


--
-- Name: idx_aliases_external; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_aliases_external ON public.product_aliases USING btree (external_code);


--
-- Name: idx_aliases_internal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_aliases_internal ON public.product_aliases USING btree (internal_id);


--
-- Name: idx_comments_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_parent ON public.comments USING btree ("parentId");


--
-- Name: idx_comments_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_target ON public.comments USING btree ("targetType", "targetId");


--
-- Name: idx_comments_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_user ON public.comments USING btree ("userId");


--
-- Name: idx_delivery_carrier_provider_code; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_delivery_carrier_provider_code ON public.delivery_carrier USING btree (provider_code);


--
-- Name: idx_delivery_tracking_picking_delivery_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_delivery_tracking_picking_delivery_id ON public.delivery_tracking USING btree (picking_delivery_id);


--
-- Name: idx_documents_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_deleted_at ON public.documents USING btree (deleted_at);


--
-- Name: idx_documents_device_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_device_id ON public.documents USING btree (device_id);


--
-- Name: idx_documents_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_status ON public.documents USING btree (status);


--
-- Name: idx_documents_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_type ON public.documents USING btree (type);


--
-- Name: idx_documents_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_user_id ON public.documents USING btree (user_id);


--
-- Name: idx_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_entity ON public.sync_conflicts USING btree (entity_type, entity_id);


--
-- Name: idx_entity_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_entity_lookup ON public.entity_checksums USING btree (entity_type, entity_id);


--
-- Name: idx_file_resources_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_file_resources_deleted_at ON public.file_resources USING btree (deleted_at);


--
-- Name: idx_file_resources_source_instance; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_file_resources_source_instance ON public.file_resources USING btree (source_instance);


--
-- Name: idx_full_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_full_hash ON public.entity_checksums USING btree (full_hash);


--
-- Name: idx_instance_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_instance_entity ON public.sync_metadata USING btree (instance_id, entity_type);


--
-- Name: idx_instance_route; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_instance_route ON public.sync_routes USING btree (instance_id, route_url);


--
-- Name: idx_item_embeddings_hnsw; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_item_embeddings_hnsw ON public.item_embeddings USING hnsw (item_embedding public.vector_cosine_ops);


--
-- Name: idx_orders_assigned_to; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_assigned_to ON public.orders USING btree (assigned_to);


--
-- Name: idx_orders_customer_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_customer_name ON public.orders USING btree (customer_name);


--
-- Name: idx_orders_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_deleted_at ON public.orders USING btree (deleted_at);


--
-- Name: idx_orders_item_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_item_id ON public.orders USING btree (item_id);


--
-- Name: idx_orders_order_number; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_orders_order_number ON public.orders USING btree (order_number);


--
-- Name: idx_orders_order_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_order_type ON public.orders USING btree (order_type);


--
-- Name: idx_orders_product_sku; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_product_sku ON public.orders USING btree (product_sku);


--
-- Name: idx_orders_serial_number; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_serial_number ON public.orders USING btree (serial_number);


--
-- Name: idx_orders_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_status ON public.orders USING btree (status);


--
-- Name: idx_page_contexts_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_page_contexts_active ON public.page_contexts USING btree ("isActive");


--
-- Name: idx_page_contexts_page_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_page_contexts_page_id ON public.page_contexts USING btree ("pageId");


--
-- Name: idx_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pending ON public.sync_conflicts USING btree (status, created_at);


--
-- Name: idx_product_aliases_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_aliases_deleted_at ON public.product_aliases USING btree (deleted_at);


--
-- Name: idx_product_product_barcode; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_product_barcode ON public.product_product USING btree (barcode);


--
-- Name: idx_product_product_default_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_product_default_code ON public.product_product USING btree (default_code);


--
-- Name: idx_registered_devices_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_registered_devices_deleted_at ON public.registered_devices USING btree (deleted_at);


--
-- Name: idx_registered_devices_home_instance_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_registered_devices_home_instance_id ON public.registered_devices USING btree (home_instance_id);


--
-- Name: idx_relay_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_relay_lookup ON public.encrypted_sync_packets USING btree (entity_type, entity_id);


--
-- Name: idx_res_partner_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_res_partner_name ON public.res_partner USING btree (name);


--
-- Name: idx_stock_location_barcode; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_stock_location_barcode ON public.stock_location USING btree (barcode);


--
-- Name: idx_stock_location_complete_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_location_complete_name ON public.stock_location USING btree (complete_name);


--
-- Name: idx_stock_lot_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_stock_lot_name ON public.stock_lot USING btree (name);


--
-- Name: idx_stock_lot_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_lot_product_id ON public.stock_lot USING btree (product_id);


--
-- Name: idx_stock_move_line_picking_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_move_line_picking_id ON public.stock_move_line USING btree (picking_id);


--
-- Name: idx_stock_move_line_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_move_line_product_id ON public.stock_move_line USING btree (product_id);


--
-- Name: idx_stock_move_line_state; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_move_line_state ON public.stock_move_line USING btree (state);


--
-- Name: idx_stock_picking_delivery_carrier_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_picking_delivery_carrier_id ON public.stock_picking_delivery USING btree (carrier_id);


--
-- Name: idx_stock_picking_delivery_last_activity_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_picking_delivery_last_activity_at ON public.stock_picking_delivery USING btree (last_activity_at);


--
-- Name: idx_stock_picking_delivery_picking_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_stock_picking_delivery_picking_id ON public.stock_picking_delivery USING btree (picking_id);


--
-- Name: idx_stock_picking_delivery_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_picking_delivery_status ON public.stock_picking_delivery USING btree (status);


--
-- Name: idx_stock_picking_delivery_tracking_number; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_picking_delivery_tracking_number ON public.stock_picking_delivery USING btree (tracking_number);


--
-- Name: idx_stock_picking_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_stock_picking_name ON public.stock_picking USING btree (name);


--
-- Name: idx_stock_picking_state; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_picking_state ON public.stock_picking USING btree (state);


--
-- Name: idx_stock_quant_location_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_quant_location_id ON public.stock_quant USING btree (location_id);


--
-- Name: idx_stock_quant_lot_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_quant_lot_id ON public.stock_quant USING btree (lot_id);


--
-- Name: idx_stock_quant_package_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_quant_package_id ON public.stock_quant USING btree (package_id);


--
-- Name: idx_stock_quant_package_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_stock_quant_package_name ON public.stock_quant_package USING btree (name);


--
-- Name: idx_stock_quant_package_package_type_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_quant_package_package_type_id ON public.stock_quant_package USING btree (package_type_id);


--
-- Name: idx_stock_quant_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_quant_product_id ON public.stock_quant USING btree (product_id);


--
-- Name: idx_sync_history_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sync_history_deleted_at ON public.sync_history USING btree (deleted_at);


--
-- Name: idx_sync_history_instance_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sync_history_instance_id ON public.sync_history USING btree (instance_id);


--
-- Name: idx_sync_history_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sync_history_provider ON public.sync_history USING btree (provider);


--
-- Name: idx_sync_history_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sync_history_status ON public.sync_history USING btree (status);


--
-- Name: idx_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_target ON public.sync_queue USING btree (target_instance);


--
-- Name: idx_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_updated ON public.entity_checksums USING btree (last_updated);


--
-- Name: idx_user_auths_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_auths_deleted_at ON public.user_auths USING btree (deleted_at);


--
-- Name: idx_warehouse_racks_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_warehouse_racks_deleted_at ON public.warehouse_racks USING btree (deleted_at);


--
-- Name: idx_warehouse_racks_mapped_location_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_warehouse_racks_mapped_location_id ON public.warehouse_racks USING btree (mapped_location_id);


--
-- Name: idx_warehouse_racks_warehouse_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_warehouse_racks_warehouse_id ON public.warehouse_racks USING btree (warehouse_id);


--
-- Name: items_menu_item_number_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX items_menu_item_number_index ON public.items USING btree (menu_item_number);


--
-- Name: items_source_unique_identifier_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX items_source_unique_identifier_index ON public.items USING btree (source_unique_identifier);


--
-- Name: menu_layouts_is_active_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX menu_layouts_is_active_index ON public.menu_layouts USING btree (is_active);


--
-- Name: operational_log_current_log_hash_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX operational_log_current_log_hash_index ON public.operational_log USING btree (current_log_hash);


--
-- Name: operational_log_event_type_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX operational_log_event_type_index ON public.operational_log USING btree (event_type);


--
-- Name: operational_log_previous_log_hash_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX operational_log_previous_log_hash_index ON public.operational_log USING btree (previous_log_hash);


--
-- Name: operational_log_timestamp_utc_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX operational_log_timestamp_utc_index ON public.operational_log USING btree (timestamp_utc);


--
-- Name: pending_changes_auto_apply_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX pending_changes_auto_apply_at_index ON public.pending_changes USING btree (auto_apply_at);


--
-- Name: pending_changes_change_type_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX pending_changes_change_type_index ON public.pending_changes USING btree (change_type);


--
-- Name: pending_changes_priority_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX pending_changes_priority_index ON public.pending_changes USING btree (priority);


--
-- Name: pending_changes_requested_by_user_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX pending_changes_requested_by_user_id_index ON public.pending_changes USING btree (requested_by_user_id);


--
-- Name: pending_changes_status_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX pending_changes_status_index ON public.pending_changes USING btree (status);


--
-- Name: pending_fiscal_operations_operation_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX pending_fiscal_operations_operation_id_index ON public.pending_fiscal_operations USING btree (operation_id);


--
-- Name: pending_fiscal_operations_status_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX pending_fiscal_operations_status_index ON public.pending_fiscal_operations USING btree (status);


--
-- Name: scans_instance_id_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX scans_instance_id_created_at ON public.scans USING btree (instance_id, "createdAt");


--
-- Name: scans_instance_id_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX scans_instance_id_status ON public.scans USING btree (instance_id, status);


--
-- Name: search_cache_query_text_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX search_cache_query_text_index ON public.search_cache USING btree (query_text);


--
-- Name: storno_log_approval_status_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX storno_log_approval_status_index ON public.storno_log USING btree (approval_status);


--
-- Name: storno_log_created_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX storno_log_created_at_index ON public.storno_log USING btree (created_at);


--
-- Name: storno_log_storno_type_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX storno_log_storno_type_index ON public.storno_log USING btree (storno_type);


--
-- Name: storno_log_transaction_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX storno_log_transaction_id_index ON public.storno_log USING btree (transaction_id);


--
-- Name: storno_log_user_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX storno_log_user_id_index ON public.storno_log USING btree (user_id);


--
-- Name: system_log_level_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX system_log_level_index ON public.system_log USING btree (level);


--
-- Name: system_log_timestamp_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX system_log_timestamp_index ON public.system_log USING btree ("timestamp");


--
-- Name: translation_cache_language_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX translation_cache_language_key ON public.translation_cache USING btree (language, key);


--
-- Name: translation_cache_language_last_used; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX translation_cache_language_last_used ON public.translation_cache USING btree (language, "lastUsed");


--
-- Name: translation_cache_source_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX translation_cache_source_created_at ON public.translation_cache USING btree (source, "createdAt");


--
-- Name: translation_cache_use_count; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX translation_cache_use_count ON public.translation_cache USING btree ("useCount");


--
-- Name: user_sessions_expires_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_sessions_expires_at_index ON public.user_sessions USING btree (expires_at);


--
-- Name: user_sessions_session_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_sessions_session_id_index ON public.user_sessions USING btree (session_id);


--
-- Name: user_sessions_user_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_sessions_user_id_index ON public.user_sessions USING btree (user_id);


--
-- Name: users_email_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX users_email_index ON public.users USING btree (email);


--
-- Name: users_is_active_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX users_is_active_index ON public.users USING btree (is_active);


--
-- Name: users_role_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX users_role_id_index ON public.users USING btree (role_id);


--
-- Name: users_username_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX users_username_index ON public.users USING btree (username);


--
-- Name: active_transaction_items_active_transaction_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX active_transaction_items_active_transaction_id_idx ON tenant_100002.active_transaction_items USING btree (active_transaction_id);


--
-- Name: active_transactions_business_date_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX active_transactions_business_date_idx ON tenant_100002.active_transactions USING btree (business_date);


--
-- Name: active_transactions_resolution_status_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX active_transactions_resolution_status_idx ON tenant_100002.active_transactions USING btree (resolution_status);


--
-- Name: active_transactions_status_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX active_transactions_status_idx ON tenant_100002.active_transactions USING btree (status);


--
-- Name: active_transactions_uuid_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX active_transactions_uuid_idx ON tenant_100002.active_transactions USING btree (uuid);


--
-- Name: categories_source_unique_identifier_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX categories_source_unique_identifier_idx ON tenant_100002.categories USING btree (source_unique_identifier);


--
-- Name: daily_log_archives_business_date_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX daily_log_archives_business_date_idx ON tenant_100002.daily_log_archives USING btree (business_date);


--
-- Name: dsfinvk_vat_mapping_dsfinvk_ust_schluessel_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX dsfinvk_vat_mapping_dsfinvk_ust_schluessel_idx ON tenant_100002.dsfinvk_vat_mapping USING btree (dsfinvk_ust_schluessel);


--
-- Name: export_jobs_job_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX export_jobs_job_id_idx ON tenant_100002.export_jobs USING btree (job_id);


--
-- Name: fiscal_log_current_log_hash_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX fiscal_log_current_log_hash_idx ON tenant_100002.fiscal_log USING btree (current_log_hash);


--
-- Name: fiscal_log_event_type_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX fiscal_log_event_type_idx ON tenant_100002.fiscal_log USING btree (event_type);


--
-- Name: fiscal_log_log_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX fiscal_log_log_id_idx ON tenant_100002.fiscal_log USING btree (log_id);


--
-- Name: fiscal_log_previous_log_hash_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX fiscal_log_previous_log_hash_idx ON tenant_100002.fiscal_log USING btree (previous_log_hash);


--
-- Name: fiscal_log_timestamp_utc_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX fiscal_log_timestamp_utc_idx ON tenant_100002.fiscal_log USING btree (timestamp_utc);


--
-- Name: fiscal_log_transaction_number_tse_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX fiscal_log_transaction_number_tse_idx ON tenant_100002.fiscal_log USING btree (transaction_number_tse);


--
-- Name: item_embeddings_item_embedding_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX item_embeddings_item_embedding_idx ON tenant_100002.item_embeddings USING hnsw (item_embedding public.vector_cosine_ops);


--
-- Name: items_menu_item_number_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX items_menu_item_number_idx ON tenant_100002.items USING btree (menu_item_number);


--
-- Name: items_source_unique_identifier_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX items_source_unique_identifier_idx ON tenant_100002.items USING btree (source_unique_identifier);


--
-- Name: menu_layouts_is_active_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX menu_layouts_is_active_idx ON tenant_100002.menu_layouts USING btree (is_active);


--
-- Name: operational_log_current_log_hash_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX operational_log_current_log_hash_idx ON tenant_100002.operational_log USING btree (current_log_hash);


--
-- Name: operational_log_event_type_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX operational_log_event_type_idx ON tenant_100002.operational_log USING btree (event_type);


--
-- Name: operational_log_previous_log_hash_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX operational_log_previous_log_hash_idx ON tenant_100002.operational_log USING btree (previous_log_hash);


--
-- Name: operational_log_timestamp_utc_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX operational_log_timestamp_utc_idx ON tenant_100002.operational_log USING btree (timestamp_utc);


--
-- Name: pending_changes_auto_apply_at_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX pending_changes_auto_apply_at_idx ON tenant_100002.pending_changes USING btree (auto_apply_at);


--
-- Name: pending_changes_change_type_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX pending_changes_change_type_idx ON tenant_100002.pending_changes USING btree (change_type);


--
-- Name: pending_changes_priority_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX pending_changes_priority_idx ON tenant_100002.pending_changes USING btree (priority);


--
-- Name: pending_changes_requested_by_user_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX pending_changes_requested_by_user_id_idx ON tenant_100002.pending_changes USING btree (requested_by_user_id);


--
-- Name: pending_changes_status_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX pending_changes_status_idx ON tenant_100002.pending_changes USING btree (status);


--
-- Name: pending_fiscal_operations_operation_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX pending_fiscal_operations_operation_id_idx ON tenant_100002.pending_fiscal_operations USING btree (operation_id);


--
-- Name: pending_fiscal_operations_status_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX pending_fiscal_operations_status_idx ON tenant_100002.pending_fiscal_operations USING btree (status);


--
-- Name: search_cache_query_text_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX search_cache_query_text_idx ON tenant_100002.search_cache USING btree (query_text);


--
-- Name: storno_log_approval_status_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX storno_log_approval_status_idx ON tenant_100002.storno_log USING btree (approval_status);


--
-- Name: storno_log_created_at_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX storno_log_created_at_idx ON tenant_100002.storno_log USING btree (created_at);


--
-- Name: storno_log_storno_type_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX storno_log_storno_type_idx ON tenant_100002.storno_log USING btree (storno_type);


--
-- Name: storno_log_transaction_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX storno_log_transaction_id_idx ON tenant_100002.storno_log USING btree (transaction_id);


--
-- Name: storno_log_user_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX storno_log_user_id_idx ON tenant_100002.storno_log USING btree (user_id);


--
-- Name: system_log_level_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX system_log_level_idx ON tenant_100002.system_log USING btree (level);


--
-- Name: system_log_timestamp_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX system_log_timestamp_idx ON tenant_100002.system_log USING btree ("timestamp");


--
-- Name: translation_cache_language_key_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE UNIQUE INDEX translation_cache_language_key_idx ON tenant_100002.translation_cache USING btree (language, key);


--
-- Name: translation_cache_language_lastUsed_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX "translation_cache_language_lastUsed_idx" ON tenant_100002.translation_cache USING btree (language, "lastUsed");


--
-- Name: translation_cache_source_createdAt_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX "translation_cache_source_createdAt_idx" ON tenant_100002.translation_cache USING btree (source, "createdAt");


--
-- Name: translation_cache_useCount_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX "translation_cache_useCount_idx" ON tenant_100002.translation_cache USING btree ("useCount");


--
-- Name: user_sessions_expires_at_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX user_sessions_expires_at_idx ON tenant_100002.user_sessions USING btree (expires_at);


--
-- Name: user_sessions_session_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX user_sessions_session_id_idx ON tenant_100002.user_sessions USING btree (session_id);


--
-- Name: user_sessions_user_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX user_sessions_user_id_idx ON tenant_100002.user_sessions USING btree (user_id);


--
-- Name: users_email_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX users_email_idx ON tenant_100002.users USING btree (email);


--
-- Name: users_is_active_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX users_is_active_idx ON tenant_100002.users USING btree (is_active);


--
-- Name: users_role_id_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX users_role_id_idx ON tenant_100002.users USING btree (role_id);


--
-- Name: users_username_idx; Type: INDEX; Schema: tenant_100002; Owner: -
--

CREATE INDEX users_username_idx ON tenant_100002.users USING btree (username);


--
-- Name: active_transaction_items active_transaction_items_active_transaction_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transaction_items
    ADD CONSTRAINT active_transaction_items_active_transaction_id_foreign FOREIGN KEY (active_transaction_id) REFERENCES public.active_transactions(id) ON DELETE CASCADE;


--
-- Name: active_transaction_items active_transaction_items_item_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transaction_items
    ADD CONSTRAINT active_transaction_items_item_id_foreign FOREIGN KEY (item_id) REFERENCES public.items(id) ON DELETE RESTRICT;


--
-- Name: active_transaction_items active_transaction_items_parent_transaction_item_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transaction_items
    ADD CONSTRAINT active_transaction_items_parent_transaction_item_id_foreign FOREIGN KEY (parent_transaction_item_id) REFERENCES public.active_transaction_items(id);


--
-- Name: active_transactions active_transactions_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.active_transactions
    ADD CONSTRAINT active_transactions_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: ai_chat_messages ai_chat_messages_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_chat_messages
    ADD CONSTRAINT "ai_chat_messages_userId_fkey" FOREIGN KEY ("userId") REFERENCES public.user_auth(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: branches branches_company_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.branches
    ADD CONSTRAINT branches_company_id_foreign FOREIGN KEY (company_id) REFERENCES public.companies(id) ON DELETE CASCADE;


--
-- Name: categories categories_parent_category_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT categories_parent_category_id_foreign FOREIGN KEY (parent_category_id) REFERENCES public.categories(id) ON DELETE SET NULL;


--
-- Name: categories categories_pos_device_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT categories_pos_device_id_foreign FOREIGN KEY (pos_device_id) REFERENCES public.pos_devices(id) ON DELETE CASCADE;


--
-- Name: comments comments_parentId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT "comments_parentId_fkey" FOREIGN KEY ("parentId") REFERENCES public.comments(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comments comments_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT "comments_userId_fkey" FOREIGN KEY ("userId") REFERENCES public.user_auth(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: dsfinvk_locations dsfinvk_locations_pos_device_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_locations
    ADD CONSTRAINT dsfinvk_locations_pos_device_id_foreign FOREIGN KEY (pos_device_id) REFERENCES public.pos_devices(id) ON DELETE SET NULL;


--
-- Name: dsfinvk_tse dsfinvk_tse_pos_device_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dsfinvk_tse
    ADD CONSTRAINT dsfinvk_tse_pos_device_id_foreign FOREIGN KEY (pos_device_id) REFERENCES public.pos_devices(id) ON DELETE CASCADE;


--
-- Name: export_jobs export_jobs_created_by_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_jobs
    ADD CONSTRAINT export_jobs_created_by_foreign FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: fiscal_log fiscal_log_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fiscal_log
    ADD CONSTRAINT fiscal_log_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: delivery_tracking fk_delivery_tracking_picking_delivery; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.delivery_tracking
    ADD CONSTRAINT fk_delivery_tracking_picking_delivery FOREIGN KEY (picking_delivery_id) REFERENCES public.stock_picking_delivery(id);


--
-- Name: orders fk_orders_assigned_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT fk_orders_assigned_user FOREIGN KEY (assigned_to) REFERENCES public.user_auths(id);


--
-- Name: stock_lot fk_product_product_stock_lots; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_lot
    ADD CONSTRAINT fk_product_product_stock_lots FOREIGN KEY (product_id) REFERENCES public.product_product(id);


--
-- Name: stock_location fk_stock_location_children; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_location
    ADD CONSTRAINT fk_stock_location_children FOREIGN KEY (location_id) REFERENCES public.stock_location(id);


--
-- Name: stock_picking_delivery fk_stock_picking_delivery_carrier; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_picking_delivery
    ADD CONSTRAINT fk_stock_picking_delivery_carrier FOREIGN KEY (carrier_id) REFERENCES public.delivery_carrier(id);


--
-- Name: stock_picking_delivery fk_stock_picking_delivery_picking; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_picking_delivery
    ADD CONSTRAINT fk_stock_picking_delivery_picking FOREIGN KEY (picking_id) REFERENCES public.stock_picking(id);


--
-- Name: stock_quant_package fk_stock_quant_package_package_type; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_quant_package
    ADD CONSTRAINT fk_stock_quant_package_package_type FOREIGN KEY (package_type_id) REFERENCES public.stock_package_type(id);


--
-- Name: warehouse_racks fk_warehouse_racks_mapped_location; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.warehouse_racks
    ADD CONSTRAINT fk_warehouse_racks_mapped_location FOREIGN KEY (mapped_location_id) REFERENCES public.stock_location(id);


--
-- Name: warehouse_racks fk_warehouse_racks_warehouse; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.warehouse_racks
    ADD CONSTRAINT fk_warehouse_racks_warehouse FOREIGN KEY (warehouse_id) REFERENCES public.stock_location(id);


--
-- Name: item_embeddings item_embeddings_item_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.item_embeddings
    ADD CONSTRAINT item_embeddings_item_id_fkey FOREIGN KEY (item_id) REFERENCES public.items(id) ON DELETE CASCADE;


--
-- Name: items items_associated_category_unique_identifier_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.items
    ADD CONSTRAINT items_associated_category_unique_identifier_foreign FOREIGN KEY (associated_category_unique_identifier) REFERENCES public.categories(id) ON DELETE CASCADE;


--
-- Name: items items_pos_device_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.items
    ADD CONSTRAINT items_pos_device_id_foreign FOREIGN KEY (pos_device_id) REFERENCES public.pos_devices(id) ON DELETE CASCADE;


--
-- Name: operational_log operational_log_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operational_log
    ADD CONSTRAINT operational_log_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: pending_changes pending_changes_requested_by_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_changes
    ADD CONSTRAINT pending_changes_requested_by_user_id_foreign FOREIGN KEY (requested_by_user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: pending_changes pending_changes_reviewed_by_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_changes
    ADD CONSTRAINT pending_changes_reviewed_by_user_id_foreign FOREIGN KEY (reviewed_by_user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: pos_devices pos_devices_branch_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pos_devices
    ADD CONSTRAINT pos_devices_branch_id_foreign FOREIGN KEY (branch_id) REFERENCES public.branches(id) ON DELETE CASCADE;


--
-- Name: registered_devices registered_devices_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.registered_devices
    ADD CONSTRAINT registered_devices_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.eckwms_instances(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: rma_requests rma_requests_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rma_requests
    ADD CONSTRAINT "rma_requests_userId_fkey" FOREIGN KEY ("userId") REFERENCES public.user_auth(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: scans scans_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.scans
    ADD CONSTRAINT scans_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.eckwms_instances(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: storno_log storno_log_approved_by_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storno_log
    ADD CONSTRAINT storno_log_approved_by_user_id_foreign FOREIGN KEY (approved_by_user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: storno_log storno_log_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storno_log
    ADD CONSTRAINT storno_log_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: tenant_access_keys tenant_access_keys_company_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tenant_access_keys
    ADD CONSTRAINT tenant_access_keys_company_id_foreign FOREIGN KEY (company_id) REFERENCES public.companies(id) ON DELETE CASCADE;


--
-- Name: user_sessions user_sessions_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: users users_pos_device_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pos_device_id_foreign FOREIGN KEY (pos_device_id) REFERENCES public.pos_devices(id) ON DELETE SET NULL;


--
-- Name: users users_role_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_role_id_foreign FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE RESTRICT;


--
-- PostgreSQL database dump complete
--

\unrestrict kdQp12cgTM0mCNk7UfgWnsDoR4xjkSiN94fB5T8cKTaDhL3yZdFpaFMnu21ooyT

